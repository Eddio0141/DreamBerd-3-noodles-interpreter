use pest::iterators::Pairs;

use super::parser::Rule;

#[derive(Debug)]
/// An abstract syntax tree that represents a Dreamberd program
pub struct Ast {
    pub statements: Vec<Statement>,
}

impl Ast {
    /// Parse a pest parse tree into an AST
    pub fn parse(pairs: Pairs<'_, Rule>) -> Self {
        todo!()
    }
}

#[derive(Debug)]
/// Single statement that does something
pub struct Statement {}
