use std::borrow::Cow;

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    prelude::Wrapper,
    Interpreter,
};

pub fn assert(_interpreter: &Interpreter, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(Error::InvalidArgumentCount {
            expected: 1,
            got: args.len(),
        });
    }

    let arg = &args[0];
    match arg.as_ref() {
        Value::Boolean(true) => Ok(Value::Undefined),
        Value::Boolean(false) => Err(Error::RuntimeException("Assertion failed".to_string())),
        _ => Err(Error::RuntimeException(
            "Assertion failed, not a boolean".to_string(),
        )),
    }
}
