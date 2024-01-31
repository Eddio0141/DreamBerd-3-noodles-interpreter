//! Module containing the standard library functions

use std::collections::HashMap;

use crate::{interpreter::runtime::state::FunctionVariant, Interpreter};

use super::{
    state::{Function, NativeFunc},
    value::{Object, Value, PROTO_PROP},
};

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

    // Object.Prototype
    let obj_proto = Object::new_raw(HashMap::from([(
        PROTO_PROP.to_string(),
        Value::Object(None),
    )]));

    // Object
    let obj = Object::new_raw(HashMap::from([("prototype".to_string(), obj_proto.into())]));

    interpreter.state.add_var("Object", obj.into(), 0)
}
