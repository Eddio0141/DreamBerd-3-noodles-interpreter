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

impl<'a> Analysis<'a> {
    /// Does a static analysis of code
    pub fn analyze(code_original: &str) -> Self {
        let (mut code, mut line_count) = eat_whitespace(code_original);

        // so far found functions
        let mut hoisted_funcs = Vec::new();
        let mut push_func = |identifier, arg_count, hoisted_line, body_location, life_time| {
            // TODO fix hoisted_line because this is invalid
            if let Some(life_time) = life_time {
                if !is_valid_lifetime(life_time) {
                    return;
                }
            }

            let func = FunctionInfo {
                identifier,
                arg_count,
                hoisted_line,
                body_location,
            };

            hoisted_funcs.push(func);
        };

        let mut pending_ws_skip = 0;

        // eat until terminator that isn't a function
        let mut eat_until_real_term = || {
            if let (Some(term), eaten_code, ws_count) = eat_chunks_until_term_in_chunk(code) {
                pending_ws_skip += ws_count;
                code = eaten_code;
            }
        };

        let mut eat_chunk = |code: &mut &str| -> Option<&str> {
            let (chunk, code_eaten, ws_skip) = parsers::eat_chunk(code);
            *code = code_eaten;
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

            let Some(chunk) = eat_chunk(&mut code) else {
                break;
            };

            // check for function definition
            // identifier will be separate from => as well as expression / block
            //
            // function ident arg1, arg2, ... => (expression ! | { block })
            // function ident arg1,arg2=>(expression! | {block})
            // function ident=>(expression! | {block})
            if is_function_header(chunk) {
                // is a function definition, currently at `ident` part
                // now should be the identifier
                let Some(chunk) = eat_chunk(&mut code) else {
                    // no identifier, and this is the end of the code
                    break;
                };

                let (identifier, chunk) = if chunk.starts_with("=>") {
                    // its missing an identifier, so we treat the arrow as the identifier
                    let next_chunk = eat_chunk(&mut code);
                    (chunk, next_chunk)
                } else {
                    match chunk.find("=>") {
                        Some(index) => (&chunk[..index], Some(&chunk[index..])), // identifier and what comes after
                        None => {
                            let next_chunk = eat_chunk(&mut code);
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
                            let next_chunk = eat_chunk(&mut code);
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
                            let next_chunk = eat_chunk(&mut code);
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
                        let next_chunk = eat_chunk(&mut code);

                        match next_chunk {
                            Some(next_chunk) => next_chunk,
                            None => break,
                        }
                    } else {
                        chunk
                    }
                };

                let index = code_original.len() - code.len();

                // block
                if chunk.starts_with('{') {
                    // TODO block
                } else {
                    // expression
                    eat_until_real_term();
                }

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

        Self { hoisted_funcs }
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
