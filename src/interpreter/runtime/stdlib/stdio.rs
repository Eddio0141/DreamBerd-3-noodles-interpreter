use crate::{
    interpreter::runtime::{error::Error, value::Value},
    Interpreter,
};

pub fn print(interpreter: &Interpreter, args: Vec<Value>) -> Result<Value, Error> {
    let mut stdout = interpreter.stdout.borrow_mut();
    for arg in args {
        writeln!(stdout, "{}", arg).map_err(|err| Error::RuntimeException(err.to_string()))?;
    }

    Ok(Value::Undefined)
}
