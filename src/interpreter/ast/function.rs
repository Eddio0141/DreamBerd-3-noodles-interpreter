//! Contains function related structures

use pest::iterators::Pair;

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::interpreter::InterpreterState;

use super::expression::Expression;
use super::Rule;

#[derive(Debug)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: Vec<Expression<'a>>,
}

impl<'a> From<Pair<'a, super::Rule>> for FunctionCall<'a> {
    fn from(value: Pair<'a, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str();

        let args = if let Some(value) = value.peek() {
            let mut args = Vec::new();

            for arg in value.into_inner() {
                if arg.as_rule() != Rule::expression {
                    continue;
                }

                args.push(Expression::from(arg));
            }

            args
        } else {
            Vec::new()
        };

        Self { name, args }
    }
}

impl<'a> FunctionCall<'a> {
    pub fn eval(&self, interpreter: &InterpreterState<'a>) -> Result<Value, Error> {
        interpreter.invoke_func(
            self.name,
            self.args
                .iter()
                .map(|arg| arg.eval(interpreter))
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}
