use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::runtime::stdlib::object;

use super::Value;

pub const PROTO_PROP: &str = "__proto__";

pub type ObjectRef = Arc<Mutex<Object>>;

#[derive(Debug, Clone)]
pub struct Object {
    properties: HashMap<String, Value>,
}

impl Object {
    /// Creates a new object with the default prototype
    pub fn new(mut properties: HashMap<String, Value>) -> Self {
        if properties.contains_key(PROTO_PROP) {
            return Self { properties };
        }

        // unwrap shouldn't fail as variables can't be deleted
        // TODO: prototype should be const when that's implemented, or somehow be readonly
        properties.insert(
            PROTO_PROP.to_string(),
            Arc::clone(&object::PROTOTYPE).into(),
        );

        Self { properties }
    }

    pub fn new_empty(properties: HashMap<String, Value>) -> Self {
        Self { properties }
    }

    pub fn get_property(&self, key: &str) -> Option<Value> {
        if let Some(value) = self.properties.get(key) {
            Some(value.to_owned())
        } else if key == PROTO_PROP {
            None
        } else {
            // prototype chain
            let Some(value) = self.properties.get(PROTO_PROP) else {
                return None;
            };

            let Value::Object(Some(value)) = value else {
                return None;
            };

            let obj = value.lock().unwrap();
            obj.get_property(key)
        }
    }

    pub fn set_property(&mut self, key: &str, value: Value) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn array_obj_iter(&self) -> ArrayObjIter {
        ArrayObjIter {
            obj: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct ArrayObjIter<'a> {
    obj: &'a Object,
    index: usize,
}

impl Iterator for ArrayObjIter<'_> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let index = if self.index == 0 {
            "-1".to_string()
        } else {
            (self.index - 1).to_string()
        };

        let prop = self.obj.get_property(&index);

        self.index += 1;

        prop
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.properties.is_empty() {
            return write!(f, "{{}}");
        }

        let properties = self
            .properties
            .iter()
            .map(|(key, value)| {
                let value = if let Value::String(value) = value {
                    let value = value.replace('\n', "\\n");
                    format!("\'{value}\'")
                } else {
                    let value = value.to_string();

                    // for the value, each newline after the first should be indented
                    let mut value = value.split('\n');

                    let mut lines = Vec::new();
                    if let Some(value) = value.next() {
                        lines.push(value.to_string());
                    }

                    for value in value {
                        lines.push(format!("  {}", value));
                    }

                    lines.join("\n")
                };

                format!("  {key}: {value}")
            })
            .collect::<Vec<_>>();

        write!(f, "{{\n{}\n}}", properties.join(",\n"))
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Value::Object(Some(Arc::new(Mutex::new(value))))
    }
}

impl From<ObjectRef> for Value {
    fn from(value: ObjectRef) -> Self {
        Value::Object(Some(value))
    }
}
