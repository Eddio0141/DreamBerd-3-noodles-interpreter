//! Contains structures that are uncertain until runtime

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    Interpreter,
};

use super::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
/// Either a variable, or a value, or a function call
pub struct UncertainExpr {
    identifier: String,
}

impl From<Pair<'_, Rule>> for UncertainExpr {
    fn from(value: pest::iterators::Pair<'_, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let identifier = value.next().unwrap().as_str().to_string();

        Self { identifier }
    }
}

impl UncertainExpr {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        let state = &interpreter.state;

        if let Some(value) = state.get_var(&self.identifier) {
            return Ok(value);
        }

        if let Ok(value) = state.invoke_func(interpreter, &self.identifier, Vec::new()) {
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
