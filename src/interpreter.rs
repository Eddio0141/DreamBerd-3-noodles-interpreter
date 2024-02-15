use self::evaluators::{expression::Expression, statement::StatementReturn};
use self::runtime::value::Value;
use nom::{combinator::eof, sequence::tuple, Parser};

use self::{
    evaluators::statement::Statement,
    parsers::types::Position,
    runtime::{state::InterpreterState, stdlib},
    static_analysis::Analysis,
};
use std::sync::Mutex;
use std::{fmt::Debug, io::Write};

pub mod error;
mod evaluators;
pub(crate) mod parsers;
pub mod runtime;
mod static_analysis;

/// The DreamBerd interpreter
pub struct Interpreter<'a> {
    state: InterpreterState,
    stdout: Mutex<&'a mut dyn Write>,
}

impl Debug for Interpreter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("state", &self.state)
            .finish()
    }
}

impl Interpreter<'_> {
    /// Evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn eval(&self, code: &str) -> Result<Vec<Value>, self::error::Error> {
        let analysis = Analysis::analyze(code);
        self.state.add_analysis_info(code, analysis);

        let mut code_with_pos = Position::new_with_extra(code, self);
        let args = (code, code_with_pos);

        let mut values = Vec::new();

        while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
            code_with_pos = code_after;
            let StatementReturn {
                value,
                return_value,
            } = statement.eval(args)?;

            if let Statement::Return(_) = statement {
                if let Some(return_value) = return_value {
                    values.push(return_value);
                }
                return Ok(values);
            }

            if let Some(value) = value {
                values.push(value);
            }
        }

        Ok(values)
    }

    /// Evaluate the given code but for repl
    /// This will first try to parse the code as an expression first
    pub fn eval_repl(&self, code: &str) -> Result<Vec<Value>, self::error::Error> {
        let analysis = Analysis::analyze(code);
        self.state.add_analysis_info(code, analysis);

        let mut code_with_pos = Position::new_with_extra(code, self);
        let args = (code, code_with_pos);

        let mut expr = tuple((Expression::parse, eof)).map(|(expr, _)| expr);
        if let Ok((_, expr)) = expr.parse(code_with_pos) {
            let res: Value = expr.eval(args)?.0.into_owned();
            return Ok(vec![res]);
        }

        let mut values = Vec::new();

        while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
            code_with_pos = code_after;
            let StatementReturn {
                value,
                return_value,
            } = statement.eval(args)?;

            if let Statement::Return(_) = statement {
                if let Some(return_value) = return_value {
                    values.push(return_value);
                }
                return Ok(values);
            }

            if let Some(value) = value {
                values.push(value);
            }
        }

        Ok(values)
    }

    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &str) -> Result<(), self::error::Error> {
        let mut stdout = std::io::stdout().lock();
        let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();
        stdlib::load(&interpreter);
        interpreter.eval(code)?;
        Ok(())
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
            stdout: Mutex::new(self.stdout),
            state: InterpreterState::default(),
        };
        stdlib::load(&interpreter);
        interpreter
    }
}
