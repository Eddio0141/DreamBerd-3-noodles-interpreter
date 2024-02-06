use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    runtime::value::{Object, Value, PROTO_PROP},
    Interpreter,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROTOTYPE: Arc<Mutex<Object>> = {
        let obj = Object::new_empty(HashMap::from([(
            PROTO_PROP.to_string(),
            Value::Object(None),
        )]));

        Arc::new(Mutex::new(obj))
    };
}

pub fn load(interpreter: &Interpreter) {
    // Object
    let obj = Object::new_empty(HashMap::from([(
        "prototype".to_string(),
        Arc::clone(&PROTOTYPE).into(),
    )]));

    interpreter.state.add_var("Object", obj.into(), 0);
}
