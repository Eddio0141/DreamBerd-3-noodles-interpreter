use std::fmt::Display;

use super::Value;

#[derive(Debug, Clone)]
pub struct Symbol {
    description: Option<String>,
    id: usize,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Symbol({})",
            match &self.description {
                Some(description) => description,
                None => "",
            }
        )
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> From<&'a Value> for &'a Symbol {
    fn from(value: &'a Value) -> Self {
        if let Value::Symbol(value) = value {
            value
        } else {
            unreachable!()
        }
    }
}
