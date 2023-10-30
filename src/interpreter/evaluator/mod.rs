use self::error::Error;

use super::ast::Ast;

pub mod error;

/// Evaluator for the interpreter. Contains internal state and is used to evaluate the AST
pub struct Evaluator;

impl Evaluator {
    /// Evaluate the given AST
    pub fn eval(ast: Ast) -> Result<(), Error> {
        todo!()
    }
}
