//! Module containing the standard library functions

use crate::Interpreter;

use super::state::FunctionVariant;

pub mod debug;
pub mod stdio;

pub fn load(interpreter: &Interpreter) {
    let funcs: Vec<(&str, fn(&_, _) -> _)> =
        vec![("assert", debug::assert), ("print", stdio::print)];

    for func in funcs {
        let (name, func) = func;
        interpreter
            .state
            .add_func(name, FunctionVariant::Native(func));
    }
}
