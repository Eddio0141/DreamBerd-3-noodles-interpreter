use std::{borrow::Cow, thread::sleep, time::Duration};

use crate::{
    prelude::Wrapper,
    runtime::{value::Value, Error},
    Interpreter,
};

pub fn sleep_ms(
    _interpreter: &Interpreter,
    args: Vec<Wrapper<Cow<Value>>>,
) -> Result<Value, Error> {
    let Some(ms) = args.first() else {
        return Ok(Value::Undefined);
    };

    let ms = f64::try_from(ms.0.as_ref())? as u64;

    sleep(Duration::from_millis(ms));

    Ok(Value::Undefined)
}
