//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;
#[cfg(test)]
mod tests;

use nom::{branch::*, combinator::value, Parser};
use parsers::*;

use crate::parsers::{
    types::{PosResult, Position},
    *,
};

/// Contains useful data about the code
#[derive(Debug, Clone)]
pub struct Analysis<'a> {
    /// Functions that's hoisted
    /// - This is only possible for functions that has a function assigned to it
    pub hoisted_funcs: Vec<FunctionInfo<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn analyze(input: &'a str) -> Self {
        let var_decl_func = |input: Position<'a, 'a>| -> PosResult<'a, 'a, FunctionInfo> {
            var_decl(function_expression)
                .map(|(var_decl_pos, identifier, life_time, (args, expr_pos))| {
                    FunctionInfo {
                        identifier: identifier.input,
                        arg_count: args.len(),
                        hoisted_line: match life_time {
                            Some(life_time) => match life_time {
                                LifeTime::Infinity => var_decl_pos.line, // positive infinity
                                LifeTime::Seconds(_) => var_decl_pos.line,
                                LifeTime::Lines(lines) => {
                                    let line = var_decl_pos.line;
                                    // only go backwards if lines is negative
                                    if lines.is_negative() {
                                        line.saturating_add_signed(lines)
                                    } else {
                                        line
                                    }
                                }
                            },
                            None => var_decl_pos.line,
                        },
                        body_location: expr_pos.index,
                    }
                })
                .parse(input)
        };

        let mut input = Position::new(input);
        let mut hoisted_funcs = Vec::new();

        loop {
            if input.input.is_empty() {
                break;
            }

            let (input_new, var_decl) = alt((
                value(None, ws_char),
                var_decl_func.map(Some),
                value(None, till_term),
            ))(input)
            .unwrap();

            input = input_new;

            if let Some(var_decl) = var_decl {
                hoisted_funcs.push(var_decl);
            }
        }

        Self { hoisted_funcs }
    }
}
