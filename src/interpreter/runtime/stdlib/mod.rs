//! Module containing the standard library functions

use crate::{
    interpreter::runtime::{value::Value, Error},
    Interpreter,
};

use super::state::FunctionVariant;

mod debug;
mod info;
mod stdio;

pub fn load(interpreter: &Interpreter) {
    type Func = fn(&Interpreter<'_>, Vec<&Value>) -> Result<Value, Error>;

    // funcs
    let funcs: Vec<(_, Func)> = vec![
        ("assert", debug::assert),
        ("print", stdio::print),
        ("typeof", info::get_typeof),
    ];

    for func in funcs {
        let (name, func) = func;
        interpreter
            .state
            .add_func(name, FunctionVariant::Native(func));
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
