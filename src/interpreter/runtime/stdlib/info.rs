use crate::{
    interpreter::runtime::{value::Value, Error},
    Interpreter,
};

pub fn get_typeof(_interpreter: &Interpreter, args: Vec<&Value>) -> Result<Value, Error> {
    let value = args.get(0).unwrap_or(&&Value::Undefined);

    let result = match value {
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
