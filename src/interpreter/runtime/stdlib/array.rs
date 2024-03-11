use lazy_static::lazy_static;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    interpreter::evaluators::variable::VarType,
    prelude::Wrapper,
    runtime::{
        state::FunctionVariant,
        value::{Object, ObjectRef, Value, PROTO_PROP},
        Error,
    },
    Interpreter,
};

lazy_static! {
    pub static ref PROTOTYPE: ObjectRef = {
        let array_proto = Object::new(HashMap::new());

        Arc::new(Mutex::new(array_proto))
    };
}

pub fn constructor(
    _interpreter: &Interpreter,
    args: Vec<Wrapper<Cow<Value>>>,
) -> Result<Value, Error> {
    let mut props = HashMap::from([(PROTO_PROP.to_string(), Arc::clone(&PROTOTYPE).into())]);

    if let Some(first) = args.first() {
        let first = first.as_ref();
        props.insert("-1".to_string(), first.clone());
    }
    for (i, item) in args.iter().enumerate().skip(1) {
        let item = item.as_ref().clone();
        props.insert((i - 1).to_string(), item);
    }

    let obj = Object::new(props);

    Ok(obj.into())
}

pub fn load(interpreter: &Interpreter) {
    // Array
    let array = Object::new_empty(HashMap::from([(
        "prototype".to_string(),
        Arc::clone(&PROTOTYPE).into(),
    )]));

    interpreter
        .state
        .add_var("Array", array.into(), 0, VarType::VarVar);
    interpreter
        .state
        .add_func_declare_var("Array", FunctionVariant::Native(constructor), None);
}
