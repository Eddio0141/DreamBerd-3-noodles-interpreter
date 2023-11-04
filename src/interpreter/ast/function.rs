//! Contains function related structures

use pest::iterators::Pair;

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::Interpreter;

use super::expression::Expression;
use super::Rule;

#[derive(Debug)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall {
    name: String,
    args: Vec<Expression>,
}

impl From<Pair<'_, super::Rule>> for FunctionCall {
    fn from(value: Pair<'_, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str().to_string();

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

impl FunctionCall {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(arg.eval(interpreter)?);
        }
        let args = args.iter().collect::<Vec<_>>();

        interpreter.state.invoke_func(interpreter, &self.name, args)
    }
}
