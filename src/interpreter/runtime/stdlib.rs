//! Module containing the standard library functions

use crate::{interpreter::runtime::state::FunctionVariant, Interpreter};

use super::state::NativeFunc;

pub mod array;
mod debug;
pub mod function;
mod info;
pub mod object;
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
        interpreter
            .state
            .add_func(name, FunctionVariant::Native(func), Some(arg_count));
    }

    object::load(interpreter);
    function::load(interpreter);
    array::load(interpreter);
}
