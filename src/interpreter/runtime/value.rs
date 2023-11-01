use std::{fmt::Display, ops::Not};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// A value that is of a certain type
pub enum Value {
    Number(f64),
    Boolean(bool),
    Undefined,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Undefined => write!(f, "undefined"),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        Value::Boolean(!(bool::from(self)))
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(num) => num != 0.0,
            Value::Boolean(value) => value,
            Value::Undefined => false,
        }
    }
}
