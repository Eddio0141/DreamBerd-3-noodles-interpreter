use self::evaluators::expression::AtomValue;
use self::evaluators::{expression::Expression, statement::StatementReturn};
use self::runtime::value::Value;
use nom::combinator::verify;
use nom::{combinator::eof, sequence::tuple, Parser};

use self::{
    evaluators::statement::Statement,
    parsers::types::Position,
    runtime::{state::InterpreterState, stdlib},
    static_analysis::Analysis,
};
use std::fmt::Debug;

pub mod error;
mod evaluators;
pub(crate) mod parsers;
pub mod runtime;
mod static_analysis;

/// The DreamBerd interpreter
pub struct Interpreter {
    state: InterpreterState,
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("state", &self.state)
            .finish()
    }
}

impl Interpreter {
    /// Evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn eval(&self, code: &str) -> Result<Vec<Value>, self::error::Error> {
        // TODO: this is terrible
        let reverse_code = code.lines().rev().collect::<String>();
        let total_lines = code.lines().count();

        let analysis = Analysis::analyze(code);
        self.state.add_analysis_info(analysis);

        let binding = (self, code);
        let mut code_with_pos = Position::new_with_extra(code, &binding);

        let mut values = Vec::new();

        while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
            // TODO: merge with below
            let StatementReturn {
                value,
                return_value,
            } = statement.eval(code_after)?;

            // TODO: remove this later maybe too
            // if let Some(new_pos) = new_pos {
            //     code_with_pos = new_pos;
            // }

            if let Statement::Return(_) = statement {
                if let Some(return_value) = return_value {
                    values.push(return_value);
                }
                return Ok(values);
            }

            if let Some(value) = value {
                values.push(value);
            }

            // reverse?
            if matches!(statement, Statement::Reverse(_)) {
                let reverse = *self.state.exec_reverse.lock().unwrap();
                // TODO: check
                code_with_pos.index = code.len() - code_with_pos.index;
                code_with_pos.line = total_lines - code_with_pos.line;
                code_with_pos.input = if reverse {
                    &reverse_code[code_with_pos.index..]
                } else {
                    &code[code_with_pos.index..]
                };
                continue;
            }

            code_with_pos = code_after;
        }

        Ok(values)
    }

    /// Evaluate the given code but for repl
    /// This will first try to parse the code as an expression first
    pub fn eval_repl(&self, code: &str) -> Result<Vec<Value>, self::error::Error> {
        let analysis = Analysis::analyze(code);
        self.state.add_analysis_info(analysis);

        let binding = (self, code);
        let mut code_with_pos = Position::new_with_extra(code, &binding);

        let mut expr = verify(tuple((Expression::parse, eof)).map(|(expr, _)| expr), |e| {
            // dont allow strings as they could be implicit strings
            // let statement handle it
            if let Expression::Atom(atom) = e {
                if let AtomValue::Value(value) = &atom.value {
                    matches!(value, Value::String(_))
                } else {
                    true
                }
            } else {
                true
            }
        });
        if let Ok((_, expr)) = expr.parse(code_with_pos) {
            let res: Value = expr.eval(code_with_pos)?.0.into_owned();
            return Ok(vec![res]);
        }

        let mut values = Vec::new();

        while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
            code_with_pos = code_after;
            let StatementReturn {
                value,
                return_value,
            } = statement.eval(code_with_pos)?;

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
        let interpreter = Self::new();
        interpreter.eval(code)?;
        Ok(())
    }

    pub fn new() -> Self {
        let interpreter = Self {
            state: InterpreterState::default(),
        };
        stdlib::load(&interpreter);
        interpreter
    }
}
