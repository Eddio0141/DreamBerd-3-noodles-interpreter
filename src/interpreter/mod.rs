use std::{cell::RefCell, io::Write};

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

pub struct Interpreter<'a> {
    state: InterpreterState<'a>,
    stdout: RefCell<&'a mut dyn Write>,
}

impl<'a> Interpreter<'a> {
    pub fn eval(&self, code: &'a str) -> Result<(), self::error::Error> {
        let parsed = PestParser::parse(Rule::program, code).map_err(Box::new)?;
        let ast = dbg!(Ast::parse(parsed));
        stdlib::load(self);
        ast.eval(self)?;
        Ok(())
    }

    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &'a str) -> Result<(), self::error::Error> {
        let mut stdout = std::io::stdout().lock();
        let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();
        interpreter.eval(code)
    }
}

/// A builder for the interpreter
pub struct InterpreterBuilder<'a> {
    stdout: &'a mut dyn Write,
}

impl<'a> InterpreterBuilder<'a> {
    /// Create a new interpreter builder
    /// - This starts from defining the stdout since it is required
    pub fn with_stdout(stdout: &'a mut dyn Write) -> Self {
        Self { stdout }
    }

    pub fn build(self) -> Interpreter<'a> {
        Interpreter {
            stdout: RefCell::new(self.stdout),
            state: InterpreterState::default(),
        }
    }
}
