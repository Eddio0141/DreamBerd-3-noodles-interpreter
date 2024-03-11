//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;
#[cfg(test)]
mod tests;

use nom::{
    branch::*, character::complete::char, combinator::value, multi::many1, sequence::tuple, Parser,
};
use parsers::*;

use crate::parsers::{types::Position, *};

/// Contains useful data about the code
#[derive(Debug, Clone)]
pub struct Analysis {
    /// Functions that's hoisted
    /// - This is only possible for functions that has a function assigned to it
    pub hoisted_vars: Vec<HoistedVarInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Information for a function
/// # Note
/// - This only applies for functions defined with `function` keyword and functions assigned to a variable
pub struct HoistedVarInfo {
    pub identifier: String,
    /// Index of the line where the function will become usable
    pub hoisted_line: usize,
    /// Location where the expression is located at
    pub expr_index: usize,
}

impl Analysis {
    /// Does a static analysis of code
    pub fn analyze(input: &str) -> Self {
        let mut input = Position::new(input);
        let mut hoisted_vars = Vec::new();

        loop {
            if input.input.is_empty() {
                break;
            }

            let (input_new, var_decl) = alt((
                value(None, ws_char),
                var_decl.map(Some),
                value(None, tuple((till_term, ws, many1(char('!'))))),
            ))(input)
            .unwrap();

            input = input_new;

            if let Some(var_decl) = var_decl {
                hoisted_vars.push(var_decl);
            }
        }

        Self { hoisted_vars }
    }
}
