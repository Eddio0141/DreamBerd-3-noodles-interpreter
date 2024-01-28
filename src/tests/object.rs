use crate::{interpreter, runtime, Interpreter};

#[test]
fn obj_initialiser() {
    let code = r#"
var var foo = { a: 1, b: 2 }!
var var foo_type = typeof foo!
assert foo_type == "object"!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_initialiser_single() {
    let code = r#"
var var foo = { a: 1 }!
var var foo_type = typeof foo!
assert foo_type == "object"!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_initialiser_properties() {
    let code = r#"
var var foo = { a : 1, b : 2 }!
assert(foo.a == 1)!
assert(foo.b == 2)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_initialiser_properties_minimal() {
    let code = r#"
var var foo = {a:1,b:2}!
assert(foo.a == 1)!
assert(foo.b == 2)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_initialiser_dupe() {
    let code = r#"
var var foo = {a: 1, b: 2, a: 3}!
assert(foo.a == 3)!
assert(foo.b == 2)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_initialiser_bracket_notation() {
    let code = r#"
var var foo = { a : 1, b : 2 }!
assert(foo["a"] == 1)!
assert(foo["b"] == 2)!
assert(foo[a] == 1)!
assert(foo[b] == 2)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_protochain() {
    let code = r#"
var var foo = { __proto__: {a: 1} }!
assert(foo.a === 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_protochain2() {
    let code = r#"
var var foo = { __proto__: {a: 1}, a: 2 }!
assert(foo.a === 2)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn obj_null_cant_read() {
    let code = r#"
var var foo = null!
print foo.a!
"#;
    let err = Interpreter::new_eval(code).unwrap_err();
    assert!(matches!(
        err,
        interpreter::error::Error::EvalError(runtime::Error::Type(_))
    ));
}
