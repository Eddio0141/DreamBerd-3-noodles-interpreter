use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;

use crate::{runtime::value::Object, Interpreter};

lazy_static! {
    pub static ref PROTOTYPE: Arc<Mutex<Object>> = {
        // Function.prototype
        let func_proto = Object::new(HashMap::new());

        Arc::new(Mutex::new(func_proto))
    };
}

pub fn load(interpreter: &Interpreter) {
    // Function
    let func = Object::new_empty(HashMap::from([(
        "prototype".to_string(),
        Arc::clone(&PROTOTYPE).into(),
    )]));

    interpreter.state.add_var("Function", func.into(), 0);
}
