use std::borrow::Cow;

use crate::{prelude::Wrapper, runtime};

use super::Value;

impl TryFrom<&Value> for f64 {
    type Error = runtime::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        f64::try_from(Wrapper(Cow::Borrowed(value)))
    }
}

impl TryFrom<Value> for f64 {
    type Error = runtime::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        f64::try_from(Wrapper(Cow::Owned(value)))
    }
}

impl<'a> TryFrom<Wrapper<Cow<'a, Value>>> for f64 {
    type Error = runtime::Error;

    fn try_from(value: Wrapper<Cow<'a, Value>>) -> Result<Self, Self::Error> {
        let num = match value.0.as_ref() {
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
