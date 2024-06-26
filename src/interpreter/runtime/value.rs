use std::{borrow::Cow, fmt::Display, ops::*, sync::Arc};

use crate::{
    interpreter::runtime,
    parsers::{types::Position, *},
    prelude::Wrapper,
};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*,
    number::complete::*, sequence::*, IResult, Parser,
};

mod bigint;
mod bool;
mod f64;
pub mod object;
mod symbol;

pub use object::*;
use symbol::*;

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
    /// - Objects are copied by reference, so cloning this will share the same object via `Arc`
    /// - Objects can be mutated via `Mutex`
    /// - Option is due to the fact that it can be `null`
    Object(Option<ObjectRef>),
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

    pub fn strict_eq(&self, other: &Self) -> bool {
        // different type means not equal
        if !self.same_type(other) {
            return false;
        }

        match self {
            Value::Number(value) => *value == f64::try_from(other).unwrap(),
            Value::Boolean(value) => *value == <&Value as Into<bool>>::into(other),
            Value::Undefined => {
                matches!(other, Value::Undefined) || matches!(other, Value::Object(None))
            }
            Value::BigInt(value) => *value == BigInt::try_from(other).unwrap(),
            Value::String(value) => value == <&Value as Into<&String>>::into(other),
            Value::Symbol(value) => value == <&Value as Into<&Symbol>>::into(other),
            Value::Object(value) => {
                let other = if let Value::Object(other) = other {
                    other
                } else {
                    unreachable!()
                };

                match value {
                    Some(value) => match other {
                        Some(other) => Arc::ptr_eq(value, other), // check reference
                        None => false,
                    },
                    None => other.is_none(),
                }
            }
        }
    }

    fn is_primitive(&self) -> bool {
        !matches!(self, Value::Object(_))
    }

    /// Loose equality comparison for same types
    fn loose_eq_primitive_eq_type(&self, other: &Self) -> bool {
        match self {
            Value::Number(value) => value == &f64::try_from(other).unwrap(),
            Value::Boolean(value) => value == &bool::from(other),
            Value::BigInt(value) => *value == BigInt::try_from(other).unwrap(),
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
                        Some(other) => Arc::ptr_eq(value, other), // check reference
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

                value == other
            }
        }
    }

    pub fn parse<T>(input: Position<T>) -> IResult<Position<T>, Self> {
        let value_true = value(Value::Boolean(true), tag::<_, _, ()>("true"));
        let value_false = value(Value::Boolean(false), tag("false"));
        let value_undefined = value(Value::Undefined, tag("undefined"));
        let value_null = value(Value::Object(None), tag("null"));
        let value_bigint = tuple((
            map_res(take_till(|c| c == 'n'), |s: Position<_, _>| {
                s.input.parse::<BigInt>()
            }),
            char('n'),
        ))
        .map(|(num, _)| Value::BigInt(num));
        let value_f64 = double.map(Value::Number);

        if let Ok((input, value)) = alt((
            value_true,
            value_false,
            value_undefined,
            value_null,
            value_bigint,
            value_f64,
        ))(input)
        {
            return Ok((input, value));
        }

        let start_quote = take_while1(|c| c == '\'' || c == '"');
        let escape_quote = char('\'');
        let escape_double_quote = char('"');
        let new_line = char('n');
        let escape_char = value(
            2,
            tuple((
                char('\\'),
                alt((escape_quote, escape_double_quote, new_line)),
            )),
        );
        let string_take_check = value(1, verify(take(1usize), |s: &str| s != "'" && s != "\""));
        let string_inner = alt((escape_char, string_take_check));
        let string_inner = fold_many0(string_inner, || 0, |acc, count| acc + count);
        let (s_new, (start_quotes, string_inner)) = tuple((start_quote, string_inner))(input)?;
        let start_quotes_len = start_quotes.input.len();
        let string_inner = &input.input[start_quotes_len..start_quotes_len + string_inner];
        // check ending quotes match
        // before that, is it an empty string?
        let (start_quotes, chunk) = if string_inner.is_empty() {
            // ok halv the first quotes
            if start_quotes_len % 2 != 0 {
                // mismatched quotes
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Fail,
                )));
            }

            let (chunk, start_quotes) =
                take::<_, _, ()>(start_quotes_len / 2)(start_quotes).unwrap();
            (start_quotes, chunk)
        } else {
            let (_, chunk) = peek(chunk)(s_new)?;
            (start_quotes, chunk)
        };
        if start_quotes
            .input
            .chars()
            .rev()
            .zip(chunk.input.chars())
            .all(|(a, b)| a == b)
        {
            // convert escape chars
            let from_to = [
                ("\\n", "\n"),
                ("\\\\", "\\"),
                ("\\\'", "\'"),
                ("\\\"", "\""),
            ];
            let mut string_inner = string_inner.to_string();
            for (from, to) in from_to.iter() {
                string_inner = string_inner.replace(from, to);
            }

            let (input, _) = take(start_quotes_len)(s_new)?;
            return Ok((input, Value::String(string_inner)));
        }

        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Fail,
        )))
    }
}

impl<'a> Wrapper<Cow<'a, Value>> {
    pub fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // TODO object to primitive, [@@toPrimitive]() with "number", valueOf(), toString()
        let left = if self.is_primitive() {
            self
        } else {
            return None;
        };

        let other = if other.is_primitive() {
            other
        } else {
            return None;
        };

        if let Value::String(left) = left.as_ref() {
            if let Value::String(right) = other.as_ref() {
                // both is string, compare lexicographically
                return Some(left.cmp(right));
            }
        }

        let left_bigint;
        let left = if let Value::BigInt(left) = left.as_ref() {
            left
        } else {
            let left = f64::try_from(left.as_ref()).unwrap_or(f64::NAN);
            if left.is_nan() {
                return None;
            }
            left_bigint = BigInt::from_f64(left).unwrap();
            &left_bigint
        };

        let other_bigint;
        let other = if let Value::BigInt(other) = other.as_ref() {
            other
        } else {
            let other = f64::try_from(other.as_ref()).unwrap_or(f64::NAN);
            if other.is_nan() {
                return None;
            }
            other_bigint = BigInt::from_f64(other).unwrap();
            &other_bigint
        };

        Some(left.cmp(other))
    }

    pub fn loose_eq(&self, other: &Self) -> Result<bool, runtime::Error> {
        // handle same type situation
        if self.same_type(other) {
            return Ok(self.loose_eq_primitive_eq_type(other));
        }

        // convert object to primitive
        let left = if self.is_primitive() {
            self
        } else {
            // TODO not finished implementing obj to primitive
            return Ok(false);
        };

        let other = if other.is_primitive() {
            other
        } else {
            // TODO not finished implementing obj to primitive
            return Ok(false);
        };

        // now see if its the same primitive type and compare
        if left.same_type(other) {
            // same type, just compare
            return Ok(left.loose_eq_primitive_eq_type(other));
        }

        if matches!(left.as_ref(), Value::Symbol(_)) {
            return Ok(matches!(other.as_ref(), Value::Symbol(_)));
        }

        // is one of them bool?
        if matches!(left.as_ref(), Value::Boolean(_))
            && !matches!(other.as_ref(), Value::Boolean(_))
        {
            let left = Wrapper(Cow::<Value>::Owned(Value::Number(f64::try_from(
                left.as_ref(),
            )?)));
            return left.loose_eq(other);
        }

        if matches!(other.as_ref(), Value::Boolean(_))
            && !matches!(left.as_ref(), Value::Boolean(_))
        {
            let other = Wrapper(Cow::<Value>::Owned(Value::Number(f64::try_from(
                other.as_ref(),
            )?)));
            return left.loose_eq(&other);
        }

        let result = match (left.as_ref(), other.as_ref()) {
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
        };

        Ok(result)
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
                    Some(value) => format!("{}", value.lock().unwrap()),
                    None => "null".to_string(),
                }
            ),
            Value::Symbol(value) => write!(f, "{value}"),
        }
    }
}

impl<'a> From<&'a Value> for &'a String {
    fn from(value: &'a Value) -> Self {
        if let Value::String(value) = value {
            value
        } else {
            unreachable!()
        }
    }
}

impl<'a> Not for Wrapper<Cow<'a, Value>> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Wrapper(Cow::Owned(Value::Boolean(!(bool::from(self)))))
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

impl<'a> Add for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn add(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Wrapper(Cow::Owned(Value::Number(left + right))))
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

impl<'a> Sub for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Wrapper(Cow::Owned(Value::Number(left - right))))
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

impl<'a> Mul for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn mul(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Wrapper(Cow::Owned(Value::Number(left * right))))
    }
}

impl Div for Value {
    type Output = Result<Self, runtime::Error>;

    fn div(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        let res = if right == 0.0 {
            Value::Undefined
        } else {
            Value::Number(left / right)
        };

        Ok(res)
    }
}

impl<'a> Div for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn div(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        let res = if right == 0.0 {
            Value::Undefined
        } else {
            Value::Number(left / right)
        };

        Ok(Wrapper(Cow::Owned(res)))
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

impl<'a> Rem for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn rem(self, rhs: Self) -> Self::Output {
        let left = f64::try_from(self)?;
        let right = f64::try_from(rhs)?;

        Ok(Wrapper(Cow::Owned(Value::Number(left % right))))
    }
}

impl<'a> Neg for Wrapper<Cow<'a, Value>> {
    type Output = Result<Self, runtime::Error>;

    fn neg(self) -> Self::Output {
        let value = f64::try_from(self)?;

        Ok(Wrapper(Cow::Owned(Value::Number(-value))))
    }
}
