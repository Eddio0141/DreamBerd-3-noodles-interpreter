use std::{borrow::Cow, io::Write};

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    prelude::Wrapper,
    Interpreter,
};

pub fn print(_interpreter: &Interpreter, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
    let mut stdout = std::io::stdout();
    for arg in args {
        stdout
            .write_fmt(format_args!("{}", arg.as_ref()))
            .map_err(|err| Error::RuntimeException(err.to_string()))?;
    }
    stdout
        .flush()
        .map_err(|err| Error::RuntimeException(err.to_string()))?;

    Ok(Value::Undefined)
}
