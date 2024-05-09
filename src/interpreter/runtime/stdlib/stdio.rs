use std::{borrow::Cow, io::Write};

use crate::{
    interpreter::runtime::{error::Error, value::Value},
    prelude::Wrapper,
    Interpreter,
};

pub fn print(_interpreter: &Interpreter, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
    let mut stdout = std::io::stdout().lock();
    for arg in args {
        writeln!(stdout, "{}", arg.as_ref())
            .map_err(|err| Error::RuntimeException(err.to_string()))?;
    }

    Ok(Value::Undefined)
}

pub fn input(_interpreter: &Interpreter, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    for arg in args {
        write!(stdout, "{}", arg.as_ref())
            .map_err(|err| Error::RuntimeException(err.to_string()))?;
    }
    stdout
        .flush()
        .map_err(|err| Error::RuntimeException(err.to_string()))?;
    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .map_err(|err| Error::RuntimeException(err.to_string()))?;
    Ok(Value::String(input.trim().to_string()))
}
