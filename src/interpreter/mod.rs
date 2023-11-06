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

/// The DreamBerd interpreter
pub struct Interpreter<'a> {
    state: InterpreterState,
    stdout: RefCell<&'a mut dyn Write>,
}

impl Interpreter<'_> {
    /// Evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn eval(&self, code: &str) -> Result<(), self::error::Error> {
        let parsed = PestParser::parse(Rule::program, code).map_err(Box::new)?;
        let ast = Ast::from(parsed);
        ast.eval_global(self)?;
        Ok(())
    }

    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &str) -> Result<(), self::error::Error> {
        let mut stdout = std::io::stdout().lock();
        let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();
        stdlib::load(&interpreter);
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

    /// Build the interpreter
    pub fn build(self) -> Interpreter<'a> {
        let interpreter = Interpreter {
            stdout: RefCell::new(self.stdout),
            state: InterpreterState::default(),
            // owned_inputs: RefCell::new(Vec::new()),
        };
        stdlib::load(&interpreter);
        interpreter
    }
}
