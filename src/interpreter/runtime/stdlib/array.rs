use std::collections::HashMap;

use crate::{runtime::value::Object, Interpreter};

pub fn load(interpreter: &Interpreter) {
    // Array.prototype
    let array_proto = Object::new_empty(HashMap::new());

    // Array
    let array = Object::new_empty(HashMap::from([(
        "prototype".to_string(),
        array_proto.into(),
    )]));

    interpreter.state.add_var("Array", array.into(), 0);
}
