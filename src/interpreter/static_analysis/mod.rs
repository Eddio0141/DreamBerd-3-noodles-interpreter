//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;

use std::collections::HashMap;

use parsers::*;

/// Contains useful data about the code
#[derive(Debug, Clone)]
pub struct Analysis<'a> {
    pub global_scope: Scope<'a>,
}

/// Scope information
#[derive(Debug, Clone)]
pub struct Scope<'a> {
    /// Functions in this scope, sorted by position they exist from, including when being hoisted
    pub functions: HashMap<&'a str, FunctionInfo>,
    /// Inner scopes
    /// - They appear in the order they are defined
    pub scope: Vec<Scope<'a>>,
}

#[derive(Debug, Clone)]
/// Information for a function
/// # Note
/// - This only applies for functions defined with `function` keyword and functions assigned to a variable
/// - If the function has a time limit on how long it will exist, `time_constraint` will be true
pub struct FunctionInfo {
    pub arg_count: usize,
    pub line: usize,
    pub time_constraint: bool,
}

impl<'a> Analysis<'a> {
    /// Does a static analysis of code
    pub fn analyze(code: &str) -> Self {
        let global_scope = Self::scope_info(code, None);
        let global_scope = Self::scope_info(code, Some(global_scope));

        Self { global_scope }
    }

    /// Gets scope information
    /// - By passing in hint from the previous iteration, it will properly update the scope information
    fn scope_info<'b>(mut code: &str, hint: Option<Scope<'b>>) -> Scope<'b> {
        let (code_inner, mut line_count) = eat_whitespace(code);
        code = code_inner;

        // so far found functions
        // TODO global funcs are made by `function` keyword, rest of them are variable scope rules
        // TODO variables in function / scope are scoped to the function
        let mut funcs = HashMap::new();
        let mut funcs_hint = hint.map(|hint| hint.functions);
        let mut push_func = |identifier, args, line, life_time| {
            if funcs.contains_key(&identifier) {
                return;
            }

            if let Some(life_time) = life_time {
                if !is_valid_lifetime(life_time) {
                    return;
                }
            }

            funcs.insert(
                identifier,
                FunctionInfo {
                    arg_count: args,
                    line,
                    // time constraint, currently only has seconds
                    time_constraint: match life_time {
                        Some(life_time) => life_time.ends_with('s'),
                        None => false,
                    },
                },
            );
        };

        let function_info = |identifier| match funcs.get_key_value(&identifier) {
            Some((_, func)) => {
                if func.line < line_count {
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
                if let Some((_, function_info)) = function_info(chunk) {
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
                push_func(identifier, arg_count, line_count, None);
                continue;
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

        Scope {
            functions: funcs,
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
