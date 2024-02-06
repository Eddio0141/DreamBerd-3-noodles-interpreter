use std::borrow::Cow;

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    prelude::Wrapper,
    Interpreter,
};

pub fn print(interpreter: &Interpreter, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
    let mut stdout = interpreter.stdout.lock().unwrap();
    for arg in args {
        writeln!(stdout, "{}", arg.as_ref())
            .map_err(|err| Error::RuntimeException(err.to_string()))?;
    }
    stdout
        .flush()
        .map_err(|err| Error::RuntimeException(err.to_string()))?;

    Ok(Value::Undefined)
}
