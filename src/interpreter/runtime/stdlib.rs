//! Module containing the standard library functions

use crate::{
    interpreter::runtime::{state::FunctionVariant, value::Value},
    Interpreter,
};

use super::state::{Function, NativeFunc};

mod debug;
mod info;
mod stdio;

pub fn load(interpreter: &Interpreter) {
    // funcs
    let funcs: Vec<(_, _, NativeFunc)> = vec![
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
