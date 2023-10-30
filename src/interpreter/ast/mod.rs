use pest::iterators::Pairs;

use self::{function::FunctionCall, variable::VariableDecl};

use super::parser::Rule;

mod expression;
mod function;
mod uncertain;
mod variable;

#[derive(Debug)]
/// An abstract syntax tree that represents a Dreamberd program
pub struct Ast<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Ast<'a> {
    /// Parse a pest parse tree into an AST
    pub fn parse(pairs: Pairs<'_, Rule>) -> Self {
        todo!()
    }
}

#[derive(Debug)]
/// Single statement that does something
pub enum Statement<'a> {
    FunctionCall(FunctionCall<'a>),
    VariableDecl(VariableDecl<'a>),
}
