use std::{fmt::Display, ops::Not};

#[derive(Debug, Clone, Copy)]
/// A value that is of a certain type
pub enum Value {
    Number(f64),
    Boolean(bool),
    Undefined,
}

impl Value {
    /// Match the type of the other value
    pub fn match_type(self, other: &Self) -> Self {
        match other {
            Self::Number(_) => Self::Number(self.into()),
            Self::Boolean(_) => Self::Boolean(self.into()),
            Self::Undefined => Self::Undefined,
        }
    }
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

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(num) => num,
            Value::Boolean(value) => value as u8 as f64,
            Value::Undefined => f64::NAN,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // first, other must be the same type
        let other = other.match_type(self);

        match self {
            Value::Number(value) => *value == other.into(),
            Value::Boolean(value) => *value == other.into(),
            Value::Undefined => matches!(other, Value::Undefined),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let other = other.match_type(self);

        match self {
            Value::Number(value) => value.partial_cmp(&other.into()),
            Value::Boolean(value) => value.partial_cmp(&other.into()),
            Value::Undefined => None,
        }
    }
}
