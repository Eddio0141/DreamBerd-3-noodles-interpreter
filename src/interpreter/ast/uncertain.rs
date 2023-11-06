//! Contains structures that are uncertain until runtime

use crate::{interpreter::runtime::value::Value, Interpreter};

use super::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
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
    pub fn eval(&self, interpreter: &Interpreter) -> Value {
        let state = &interpreter.state;

        if let Some(value) = state.get_var(&self.identifier) {
            return value;
        }

        if let Ok(value) = state.invoke_func(interpreter, &self.identifier, Vec::new()) {
            return value;
        }

        // number?
        if let Ok(value) = self.identifier.parse::<f64>() {
            return Value::Number(value);
        }

        // bool?
        if let Ok(value) = self.identifier.parse::<bool>() {
            return Value::Boolean(value);
        }

        // fallback to string, strip it of quotes
        Value::String(raw_string_to_proper(&self.identifier))
    }
}

#[derive(Debug, Clone)]
/// Either a variable, or a string, or a function call
pub struct UncertainString {
    value: String,
}

impl From<Pair<'_, Rule>> for UncertainString {
    fn from(value: pest::iterators::Pair<'_, super::Rule>) -> Self {
        let value = value.into_inner().as_str().to_string();

        Self { value }
    }
}

fn raw_string_to_proper(value: &str) -> String {
    let mut value = value.trim();
    let trimmer = ['"', '\''];
    while value.len() >= 2 {
        let (first, last) = {
            let mut chars = value.chars();
            (chars.next().unwrap(), chars.last().unwrap())
        };

        if first != last {
            break;
        }

        if trimmer.contains(&first) {
            value = &value[1..value.len() - 1];
        } else {
            break;
        }
    }

    let mut value = value.to_string();

    // now make it a proper string
    let replace_escape = [("\\\"", "\""), ("\\'", "'"), ("\\\\", "\\"), ("\\n", "\n")];
    for (from, to) in replace_escape.iter() {
        value = value.replace(from, to);
    }

    value
}

impl UncertainString {
    pub fn eval(&self, interpreter: &Interpreter) -> Value {
        let state = &interpreter.state;

        if let Some(value) = state.get_var(&self.value) {
            return value;
        }

        if let Ok(value) = state.invoke_func(interpreter, &self.value, Vec::new()) {
            return value;
        }

        // fallback to string
        Value::String(raw_string_to_proper(&self.value))
    }
}
