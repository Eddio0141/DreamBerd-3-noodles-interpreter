//! Module containing the standard library functions

use super::state::{FunctionVariant, InterpreterState};

pub mod debug;

pub fn load(state: &InterpreterState) {
    let funcs = vec![("assert", debug::assert)];

    for func in funcs {
        let (name, func) = func;
        state.add_func(name, FunctionVariant::Native(func));
    }
}
