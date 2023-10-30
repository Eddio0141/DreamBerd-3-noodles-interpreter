use pest::Parser;

use crate::interpreter::{ast::Ast, evaluator::Evaluator};

use self::parser::{PestParser, Rule};

mod ast;
pub mod error;
mod evaluator;
mod parser;

pub struct Interpreter;

impl Interpreter {
    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &str) -> Result<(), self::error::Error> {
        let parsed = PestParser::parse(Rule::program, code)?;
        let ast = Ast::parse(parsed);
        Evaluator::eval(ast)?;
        Ok(())
    }
}
