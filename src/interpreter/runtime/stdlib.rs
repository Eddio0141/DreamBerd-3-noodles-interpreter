//! Module containing the standard library functions

use std::borrow::Cow;

use crate::{
    interpreter::runtime::{state::FunctionVariant, value::Value, Error},
    prelude::Wrapper,
    Interpreter,
};

use super::state::Function;

mod debug;
mod info;
mod stdio;

pub fn load(interpreter: &Interpreter) {
    type Func = fn(&Interpreter<'_>, Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error>;

    // funcs
    let funcs: Vec<(_, _, Func)> = vec![
        ("assert", 1, debug::assert),
        ("print", 1, stdio::print),
        ("typeof", 1, info::get_typeof),
    ];

    for func in funcs {
        let (name, arg_count, func) = func;
        interpreter.state.add_func(
            name,
            Function {
                arg_count,
                variant: FunctionVariant::Native(func),
            },
        );
    }

    // vars
    let vars = vec![
        ("undefined", Value::Undefined),
        ("null", Value::Object(None)),
    ];

    for (name, value) in vars {
        interpreter.state.add_var(name, value);
    }
}
