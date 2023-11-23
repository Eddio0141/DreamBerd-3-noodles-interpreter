use std::{cell::RefCell, fmt::Debug, io::Write};

use self::{
    evaluators::statement::Statement,
    parsers::types::Position,
    runtime::{state::InterpreterState, stdlib},
    static_analysis::Analysis,
};

pub mod error;
mod evaluators;
pub mod parsers;
mod runtime;
mod static_analysis;

/// The DreamBerd interpreter
pub struct Interpreter<'a> {
    state: InterpreterState,
    stdout: RefCell<&'a mut dyn Write>,
}

impl Debug for Interpreter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("state", &self.state)
            .finish()
    }
}

impl<'a> Interpreter<'a> {
    /// Evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn eval(&'a self, code: &'a str) -> Result<(), self::error::Error> {
        let analysis = Analysis::analyze(code);
        self.state.add_analysis_info(analysis);

        let mut code = Position::new_with_extra(code, self);
        while let Ok((code_after, statement)) = Statement::parse(code) {
            code = code_after;
            statement.eval(self)?;
        }

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
        };
        stdlib::load(&interpreter);
        interpreter
    }
}
