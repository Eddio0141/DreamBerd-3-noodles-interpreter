use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};

#[derive(Debug, Clone, Copy)]
/// A value that is of a certain type
pub enum Value {
    Number(f64),
    Boolean(bool),
    Undefined,
}

impl Value {
    /// Match the type of the other value
    fn match_type(self, other: &Self) -> Self {
        match other {
            // TODO is this right
            Self::Number(_) => Self::Number(self.try_into().unwrap()),
            Self::Boolean(_) => Self::Boolean(self.into()),
            Self::Undefined => Self::Undefined,
        }
    }

    fn can_be_num(&self) -> bool {
        matches!(self, Self::Number(_))
            || matches!(self, Self::Boolean(_))
            || matches!(self, Self::Undefined)
    }

    pub fn pow(self, rhs: Self) -> Self {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(
                f64::try_from(self)
                    .unwrap()
                    .powf(f64::try_from(rhs).unwrap()),
            );
        }

        unreachable!();
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

impl TryFrom<Value> for f64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let num = match value {
            Value::Number(num) => num,
            Value::Boolean(value) => value as u8 as f64,
            Value::Undefined => f64::NAN,
        };

        Ok(num)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // first, other must be the same type
        let other = other.match_type(self);

        match self {
            Value::Number(value) => *value == f64::try_from(other).unwrap(),
            Value::Boolean(value) => *value == bool::from(other),
            Value::Undefined => matches!(other, Value::Undefined),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let other = other.match_type(self);

        match self {
            Value::Number(value) => value.partial_cmp(&other.try_into().unwrap()),
            Value::Boolean(value) => value.partial_cmp(&other.into()),
            Value::Undefined => None,
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(f64::try_from(self).unwrap() + f64::try_from(rhs).unwrap());
        }

        unreachable!();
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(f64::try_from(self).unwrap() - f64::try_from(rhs).unwrap());
        }

        unreachable!();
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(f64::try_from(self).unwrap() * f64::try_from(rhs).unwrap());
        }

        unreachable!();
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(f64::try_from(self).unwrap() / f64::try_from(rhs).unwrap());
        }

        unreachable!();
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        if self.can_be_num() && rhs.can_be_num() {
            return Value::Number(f64::try_from(self).unwrap() % f64::try_from(rhs).unwrap());
        }

        unreachable!();
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.can_be_num() {
            return Value::Number(-f64::try_from(self).unwrap());
        }

        unreachable!();
    }
}
