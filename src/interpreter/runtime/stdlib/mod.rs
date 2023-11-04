//! Module containing the standard library functions

use crate::{
    interpreter::runtime::{value::Value, Error},
    Interpreter,
};

use super::state::FunctionVariant;

pub mod debug;
pub mod stdio;

pub fn load(interpreter: &Interpreter) {
    type Func = fn(&Interpreter<'_>, Vec<&Value>) -> Result<Value, Error>;

    let funcs: Vec<(_, Func)> = vec![("assert", debug::assert), ("print", stdio::print)];

    for func in funcs {
        let (name, func) = func;
        interpreter
            .state
            .add_func(name, FunctionVariant::Native(func));
    }
}
