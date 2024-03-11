use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;

use crate::{
    interpreter::evaluators::variable::VarType,
    runtime::value::{Object, ObjectRef, Value},
    Interpreter,
};

lazy_static! {
    pub static ref PROTOTYPE: ObjectRef = {
        // Function.prototype
        let func_proto = Object::new(HashMap::from([("arguments".to_string(), Value::Object(None))]));

        Arc::new(Mutex::new(func_proto))
    };
}

pub fn load(interpreter: &Interpreter) {
    // Function
    let func = Object::new_empty(HashMap::from([(
        "prototype".to_string(),
        Arc::clone(&PROTOTYPE).into(),
    )]));

    interpreter
        .state
        .add_var("Function", func.into(), 0, VarType::VarVar);
}
