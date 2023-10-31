//! Contains structures that are uncertain until runtime

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    Interpreter,
};

use super::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
/// Either a variable, or a value, or a function call
pub struct UncertainExpr<'a> {
    pub identifier: &'a str,
}

impl<'a> From<Pair<'a, Rule>> for UncertainExpr<'a> {
    fn from(value: pest::iterators::Pair<'a, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let identifier = value.next().unwrap().as_str();

        Self { identifier }
    }
}

impl<'a> UncertainExpr<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        let state = &interpreter.state;

        if let Some(value) = state.get_var(self.identifier) {
            return Ok(value);
        }

        if let Ok(value) = state.invoke_func(interpreter, self.identifier, Vec::new()) {
            return Ok(value);
        }

        // number?
        if let Ok(value) = self.identifier.parse::<f64>() {
            return Ok(Value::Number(value));
        }

        // bool?
        if let Ok(value) = self.identifier.parse::<bool>() {
            return Ok(Value::Boolean(value));
        }

        Ok(Value::Undefined)
    }
}
