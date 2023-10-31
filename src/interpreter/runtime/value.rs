use std::fmt::Display;

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
