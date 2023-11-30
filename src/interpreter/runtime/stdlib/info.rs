use std::borrow::Cow;

use crate::{
    interpreter::runtime::{value::Value, Error},
    prelude::Wrapper,
    Interpreter,
};

pub fn get_typeof(
    _interpreter: &Interpreter,
    args: Vec<Wrapper<Cow<Value>>>,
) -> Result<Value, Error> {
    let value = args
        .first()
        .unwrap_or(&Wrapper(Cow::Owned(Value::Undefined)));

    let result = match value.as_ref() {
        Value::Number(_) => "number",
        Value::Boolean(_) => "boolean",
        Value::BigInt(_) => "bigint",
        Value::String(_) => "string",
        Value::Undefined => "undefined",
        Value::Symbol(_) => "symbol",
        Value::Object(_) => "object",
    };

    Ok(Value::String(result.to_string()))
}
