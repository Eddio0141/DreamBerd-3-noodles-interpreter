use crate::interpreter::runtime::{error::Error, state::InterpreterState, value::Value};

pub fn assert(_state: &InterpreterState, args: Vec<Value>) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(Error::InvalidArgumentCount {
            expected: 1,
            got: args.len(),
        });
    }

    let arg = args[0];
    if let Value::Boolean(false) = arg {
        return Err(Error::RuntimeException("Assertion failed".to_string()));
    }

    Ok(Value::Undefined)
}
