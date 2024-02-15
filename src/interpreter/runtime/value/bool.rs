use std::borrow::Cow;

use num_bigint::BigInt;

use crate::prelude::Wrapper;

use super::Value;

impl<'a> From<Wrapper<Cow<'a, Value>>> for bool {
    fn from(value: Wrapper<Cow<'a, Value>>) -> Self {
        match value.0.as_ref() {
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
        bool::from(Wrapper(Cow::Owned(value)))
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        bool::from(Wrapper(Cow::Borrowed(value)))
    }
}
