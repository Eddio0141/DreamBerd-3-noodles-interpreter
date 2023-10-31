use pest::Parser;

use crate::interpreter::ast::Ast;

use self::{
    parser::{PestParser, Rule},
    runtime::{state::InterpreterState, stdlib},
};

mod ast;
pub mod error;
mod parser;
mod runtime;

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &str) -> Result<(), self::error::Error> {
        let parsed = PestParser::parse(Rule::program, code)?;
        let ast = Ast::parse(parsed);
        let state = InterpreterState::default();
        stdlib::load(&state);
        ast.eval(&state)?;
        Ok(())
    }
}
