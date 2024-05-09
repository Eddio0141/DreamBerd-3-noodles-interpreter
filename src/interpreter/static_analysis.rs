//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;
#[cfg(test)]
mod tests;

use nom::{
    branch::*, character::complete::char, combinator::value, multi::many0, sequence::tuple, Parser,
};
use parsers::*;

use crate::{
    parsers::{types::Position, *},
    runtime::value::Value,
};

use super::evaluators::statement::Statement;

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
    /// Location where the declaration is at
    pub decl_index: usize,
}

impl HoistedVarInfo {
    pub fn eval(&self, args: PosWithInfo) -> Option<Value> {
        let code = args.extra.1;
        let input = Position::new_with_extra(&code[self.decl_index..], args.extra);
        let (_, statement) = Statement::parse(input).unwrap();
        let Statement::VariableDecl(decl) = statement else {
            return None;
        };
        decl.expression.eval(args).ok().map(|x| x.0.into_owned())
    }
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
                value(None, tuple((till_term, ws, many0(char('!'))))),
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
