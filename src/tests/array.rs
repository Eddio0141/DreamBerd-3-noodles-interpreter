use crate::Interpreter;

#[test]
fn array_init() {
    let code = r#"
var var foo = [ 1, false, "bar" ]!
var var foo_type = typeof foo!
assert foo_type === "object"!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn array_read_normal() {
    let code = r#"
var var foo = [ 1, 2, 3 ]!
assert foo[-1] === 1!
assert foo[0] === 2!
assert foo[1] === 3!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn array_read_bracket_str() {
    let code = r#"
var var foo = [ 1, 2, 3 ]!
assert foo["-1"] === 1!
assert foo["0"] === 2!
assert foo["1"] === 3!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn array_read_oob() {
    let code = r#"
var var foo = [ 1, 2 ]!
assert foo[-2] === undefined!
assert foo[1] === undefined!
assert foo[0] === 2!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn array_read_invalid_prop_name() {
    let code = r#"
    var var foo = [ 1, 2, 3, 4 ]!
foo["02"] = "bar"!
assert foo["-1"] ;== foo["-01"]!
assert foo["-01"] === undefined!
assert foo["2"] ;== foo["02"]!
assert foo["02"] === "bar"!
"#;
    Interpreter::new_eval(code).unwrap();
}

// TODO error trying to use int for property read with dot notation
