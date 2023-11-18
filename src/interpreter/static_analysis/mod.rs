//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;

use parsers::*;

/// Contains useful data about the code
#[derive(Debug, Clone)]
pub struct Analysis<'a> {
    /// Functions that's hoisted
    /// - This is only possible for functions that has a function assigned to it
    pub hoisted_funcs: Vec<FunctionInfo<'a>>,
}

#[derive(Debug, Clone)]
/// Information for a function
/// # Note
/// - This only applies for functions defined with `function` keyword and functions assigned to a variable
pub struct FunctionInfo<'a> {
    pub identifier: &'a str,
    pub arg_count: usize,
    /// Index of the line where the function will become usable
    pub hoisted_line: usize,
    /// Where the expression / scope is located as an index
    pub body_location: usize,
}

/// Position information
pub struct Position {
    pub line: usize,
    pub index: usize,
}

impl<'a> Analysis<'a> {
    /// Does a static analysis of code
    pub fn analyze(code: &str) -> Self {
        // two passes to properly filter out functions and other stuff
        let info = Self::scope_info(code, None);
        Self::scope_info(code, Some(info))
    }

    /// Gets scope information
    /// - By passing in hint from the previous iteration, it will properly update the scope information
    fn scope_info<'b>(code_original: &str, hint: Option<Self>) -> Self {
        let (mut code, mut line_count) = eat_whitespace(code_original);

        // so far found functions
        // TODO global funcs are made by `function` keyword, rest of them are variable scope rules
        // TODO variables in function / scope are scoped to the function
        let mut funcs_global = Vec::new();
        let mut funcs_local = Vec::new();
        let mut push_func =
            |identifier, arg_count, exist_start_line, body_location, life_time, global_func| {
                if let Some(life_time) = life_time {
                    if !is_valid_lifetime(life_time) {
                        return;
                    }
                }

                let func = FunctionInfo {
                    identifier,
                    arg_count,
                    hoisted_line: exist_start_line,
                    body_location,
                };

                if global_func {
                    funcs_global.push(func);
                } else {
                    funcs_local.push(func);
                }
            };

        let function_info = |identifier| match funcs_global.get_key_value(&identifier) {
            Some((_, func)) => {
                if func.exist_start_line < line_count {
                    Some(func)
                } else {
                    None
                }
            }
            None => None,
        };

        let mut pending_ws_skip = 0;

        // eat until terminator that isn't a function
        let mut eat_until_real_term = || {
            while let (Some(term), eaten_code, ws_count) = eat_chunks_until_term_in_chunk(code) {
                pending_ws_skip += ws_count;
                code = eaten_code;
                if function_info(term).is_none() {
                    // found a terminator that isn't a function
                    break;
                }
            }
        };

        let mut eat_chunk = || -> Option<&str> {
            let (chunk, code_eaten, ws_skip) = parsers::eat_chunk(code);
            code = code_eaten;
            pending_ws_skip = ws_skip;

            chunk
        };

        let mut eat_whitespace = |code| -> &str {
            let (code, ws) = parsers::eat_whitespace(code);
            pending_ws_skip += ws;
            code
        };

        let mut scope_index = 0usize;

        // go through code
        // TODO comment
        loop {
            line_count += pending_ws_skip;
            pending_ws_skip = 0;

            let Some(chunk) = eat_chunk() else {
                break;
            };

            // check for function definition
            // identifier will be separate from => as well as expression / block
            //
            // function ident arg1, arg2, ... => (expression ! | { block })
            // function ident arg1,arg2=>(expression! | {block})
            // function ident=>(expression! | {block})
            if is_function_header(chunk) {
                // function call, not a definition
                if let Some(function_info) = function_info(chunk) {
                    if function_info.arg_count > 0 {
                        // because it has arg, it'll just pass everything into this until the terminator
                        // should be fine
                        eat_until_real_term();
                        continue;
                    }
                }

                // is a function definition, currently at `ident` part
                // now should be the identifier
                let Some(chunk) = eat_chunk() else {
                    // no identifier, and this is the end of the code
                    break;
                };

                let (identifier, chunk) = if chunk.starts_with("=>") {
                    // its missing an identifier, so we treat the arrow as the identifier
                    let next_chunk = eat_chunk();
                    (chunk, next_chunk)
                } else {
                    match chunk.find("=>") {
                        Some(index) => (&chunk[..index], Some(&chunk[index..])), // identifier and what comes after
                        None => {
                            let next_chunk = eat_chunk();
                            (chunk, next_chunk)
                        }
                    }
                };

                // either the arguments or the arrow of the end of code
                let mut chunk = match chunk {
                    Some(chunk) => chunk,
                    None => break,
                };

                // any args?
                // args must be separated by comma like `arg1, arg2, ...` and terminated by `=>`
                let arg_count = if !chunk.starts_with("=>") {
                    // just need the count
                    let mut count = 1usize;
                    // indication on end of code
                    let mut no_chunk = false;

                    // they could be all in 1 chunk
                    loop {
                        if let Some(arrow_pos) = chunk.find("=>") {
                            chunk = &chunk[arrow_pos - 2..];
                            // end
                            break;
                        }

                        // progress chunk if no comma in this chunk
                        if !chunk.contains(',') {
                            let next_chunk = eat_chunk();
                            match next_chunk {
                                Some(next_chunk) => chunk = next_chunk,
                                None => {
                                    no_chunk = true;
                                    break;
                                }
                            }
                        }

                        let Some(comma_pos) = chunk.find(',') else {
                            // no more commas, so we are done
                            break;
                        };

                        chunk = &chunk[comma_pos + 1..];
                        chunk = eat_whitespace(chunk);

                        if chunk.is_empty() {
                            let next_chunk = eat_chunk();
                            match next_chunk {
                                Some(next_chunk) => chunk = next_chunk,
                                None => {
                                    no_chunk = true;
                                    break;
                                }
                            }
                        }

                        count += 1;
                    }

                    if no_chunk {
                        // no more chunks, so we are done
                        break;
                    }

                    count
                } else {
                    0
                };

                // now should be at =>
                if !chunk.starts_with("=>") {
                    // invalid syntax, either a string or a function call, eat until terminator
                    eat_until_real_term();
                    continue;
                }

                // now progress to expression / block
                let chunk = {
                    let chunk = &chunk[2..];
                    let chunk = eat_whitespace(chunk);
                    if chunk.is_empty() {
                        let next_chunk = eat_chunk();

                        match next_chunk {
                            Some(next_chunk) => next_chunk,
                            None => break,
                        }
                    } else {
                        chunk
                    }
                };

                let index = code_original.len() - code.len();

                // expression / block
                // is this a function call?
                if function_info(chunk).is_some() || !chunk.starts_with('{') {
                    // doesn't matter if its a function without args, it will be implicit string, which will take until terminator
                    // or this is an expression
                    eat_until_real_term();
                } else {
                    // TODO block
                }

                // valid function definition
                push_func(identifier, arg_count, line_count, index, None);
                continue;
            }

            // scope?
            if chunk.starts_with("{") {
                scope_index += 1;
            }

            // TODO `var var func = arg1, arg2 => (expression ! | { block })`
            // TODO `var var func = arg1,arg2=>(expression ! | { block })`
            // TODO `var var func = =>(expression! | {block})`
            // TODO `var var func<life_time> = arg1, arg2 => (expression ! | { block })`
            match chunk {
                "var" => {}

                // maybe variable being defined
                _ => (),
            }
        }

        ScopeInfo {
            functions: funcs_global,
            // TODO
            scope: Vec::new(),
        }
    }
}

fn is_valid_lifetime(mut life_time: &str) -> bool {
    if life_time == "Infinity" {
        return true;
    }

    // check if seconds
    if life_time.ends_with('s') {
        if life_time.starts_with('-') {
            // negative seconds...
            return false;
        }

        life_time = &life_time[..life_time.len() - 1];
        return life_time.parse::<f64>().is_ok();
    }

    // lines, which can be negative so just pass as number
    life_time.parse::<f64>().is_ok()
}
