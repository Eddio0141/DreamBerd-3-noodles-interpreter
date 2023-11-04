use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    rc::Rc,
};

use crate::interpreter::runtime;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

#[derive(Debug, Clone)]
/// A value that corresponds to a ECMAScript value
pub enum Value {
    Number(f64),
    Boolean(bool),
    BigInt(BigInt),
    /// utf-16 string
    String(String),
    Undefined,
    Symbol(Symbol),
    /// - Objects are copied by reference, so cloning this will share the same object via `Rc`
    /// - Objects can be mutated via `RefCell`
    /// - Option is due to the fact that it can be `null`
    Object(Option<Rc<RefCell<Object>>>),
}

impl Value {
    pub fn pow(&self, rhs: &Self) -> Result<Self, runtime::Error> {
        Ok(Value::Number(
            f64::try_from(self)?.powf(f64::try_from(rhs)?),
        ))
    }

    fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    // fn strict_eq(&self, other: &Self) -> bool {
    //     // different type means not equal
    //     if !self.same_type(other) {
    //         return false;
    //     }

    //     todo!()
    // }

    fn is_primitive(&self) -> bool {
        !matches!(self, Value::Object(_))
    }

    /// Loose equality comparison for same types
    fn loose_eq_primitive_eq_type(&self, other: &Self) -> bool {
        match self {
            Value::Number(value) => value == &f64::try_from(other).unwrap(),
            Value::Boolean(value) => value == &bool::from(other),
            Value::BigInt(value) => value == &BigInt::try_from(other).unwrap(),
            Value::String(value) => *value == other.to_string(),
            Value::Undefined => {
                matches!(other, Value::Undefined) || matches!(other, Value::Object(None))
            }
            Value::Object(value) => {
                let other = if let Value::Object(other) = other {
                    other
                } else {
                    unreachable!()
                };

                match value {
                    Some(value) => match other {
                        Some(other) => Rc::ptr_eq(value, other), // check reference
                        None => false,
                    },
                    None => other.is_none(),
                }
            }
            Value::Symbol(value) => {
                let other = if let Value::Symbol(other) = other {
                    other
                } else {
                    unreachable!()
                };

                value.id == other.id
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // handle same type situation
        if self.same_type(other) {
            return self.loose_eq_primitive_eq_type(other);
        }

        // convert object to primitive
        let left = if self.is_primitive() {
            self
        } else {
            // TODO not finished implementing obj to primitive
            return false;
        };

        let other = if other.is_primitive() {
            other
        } else {
            // TODO not finished implementing obj to primitive
            return false;
        };

        // now see if its the same primitive type and compare
        if left.same_type(other) {
            // same type, just compare
            return left.loose_eq_primitive_eq_type(other);
        }

        if matches!(left, Value::Symbol(_)) {
            return matches!(other, Value::Symbol(_));
        }

        // is one of them bool?
        if matches!(left, Value::Boolean(_)) {
            let other = bool::from(other);
            return left == &Value::Boolean(other);
        }

        if matches!(other, Value::Boolean(_)) {
            let left: bool = bool::from(left);
            return other == &Value::Boolean(left);
        }

        match (left, other) {
            (Value::Number(left), Value::String(other)) => match other.parse::<f64>() {
                Ok(other) => *left == other,
                Err(_) => false,
            },
            (Value::Number(left), Value::BigInt(other)) => {
                if left.is_infinite() || left.is_nan() {
                    false
                } else {
                    BigInt::from_f64(*left).unwrap() == *other
                }
            }
            // TODO same behaviour as bigint constructor
            (Value::String(left), Value::BigInt(other)) => match left.parse::<BigInt>() {
                Ok(left) => left == *other,
                Err(_) => false,
            },
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Undefined => write!(f, "undefined"),
            Value::BigInt(value) => write!(f, "{value}n"),
            Value::String(value) => write!(f, "{value}"),
            Value::Object(value) => write!(
                f,
                "{}",
                match value {
                    Some(value) => format!("{}", value.borrow()),
                    None => "null".to_string(),
                }
            ),
            Value::Symbol(value) => write!(f, "{value}"),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        Value::Boolean(!(bool::from(&self)))
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        match value {
            Value::Number(num) => *num != 0.0,
            Value::Boolean(value) => *value,
            Value::Undefined => false,
            Value::BigInt(value) => *value != BigInt::from(0),
            Value::String(value) => !value.is_empty(),
            Value::Object(_) => true,
            Value::Symbol(_) => true,
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        bool::from(&value)
    }
}

impl TryFrom<&Value> for f64 {
    type Error = runtime::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let num = match value {
            Value::Number(num) => *num,
            Value::Boolean(value) => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Undefined => f64::NAN,
            Value::BigInt(_) => {
                return Err(runtime::Error::Type(
                    "Cannot convert BigInt to Number".to_string(),
                ))
            }
            Value::String(value) => value.parse().unwrap_or(f64::NAN),
            Value::Object(value) => match value {
                Some(_) => {
                    return Err(runtime::Error::Type(
                        "Not implemented object to number coercion".to_string(), // TODO this isn't right
                    ));
                }
                None => 0.0,
            },
            Value::Symbol(_) => {
                return Err(runtime::Error::Type(
                    "Cannot convert Symbol to Number".to_string(),
                ))
            }
        };

        Ok(num)
    }
}

impl TryFrom<Value> for f64 {
    type Error = runtime::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        f64::try_from(&value)
    }
}

impl TryFrom<&Value> for BigInt {
    type Error = runtime::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let num = match value {
            Value::Number(num) => BigInt::from_f64(*num).unwrap(),
            Value::Boolean(value) => {
                if *value {
                    BigInt::from(1)
                } else {
                    BigInt::from(0)
                }
            }
            Value::Undefined => {
                return Err(runtime::Error::Type(
                    "Cannot convert undefined to BigInt".to_string(),
                ))
            }
            // TODO expensive
            Value::BigInt(value) => value.clone(),
            Value::String(value) => value
                .parse()
                .map_err(|_| runtime::Error::Type("Cannot convert string to BigInt".to_string()))?,
            Value::Object(value) => {
                let err = match value {
                    Some(_) => runtime::Error::Type("Cannot convert object to BigInt".to_string()), // TODO this isn't right
                    None => runtime::Error::Type("Cannot convert null to BigInt".to_string()),
                };
                return Err(err);
            }
            Value::Symbol(_) => {
                return Err(runtime::Error::Type(
                    "Cannot convert Symbol to BigInt".to_string(),
                ))
            }
        };

        Ok(num)
    }
}

impl Add for Value {
    type Output = Result<Self, runtime::Error>;

    fn add(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Value::Number(left + right))
    }
}

impl Sub for Value {
    type Output = Result<Self, runtime::Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Value::Number(left - right))
    }
}

impl Mul for Value {
    type Output = Result<Self, runtime::Error>;

    fn mul(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Value::Number(left * right))
    }
}

impl Div for Value {
    type Output = Result<Self, runtime::Error>;

    fn div(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(if right == 0.0 {
            Value::Undefined
        } else {
            Value::Number(left / right)
        })
    }
}

impl Rem for Value {
    type Output = Result<Self, runtime::Error>;

    fn rem(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Value::Number(left % right))
    }
}

impl Neg for Value {
    type Output = Result<Self, runtime::Error>;

    fn neg(self) -> Self::Output {
        let value = f64::try_from(self)?;

        Ok(Value::Number(-value))
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    properties: HashMap<String, Value>,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.properties.is_empty() {
            return write!(f, "{{}}");
        }

        let properties = self
            .properties
            .iter()
            .map(|(key, value)| {
                let value = if let Value::String(value) = value {
                    format!("\'{value}\'")
                } else {
                    value.to_string()
                };

                format!("  {key}: {value}")
            })
            .collect::<Vec<_>>();

        write!(f, "{{\n{}\n}}", properties.join(",\n"))
    }
}

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
