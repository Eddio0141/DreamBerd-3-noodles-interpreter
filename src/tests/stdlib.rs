use crate::Interpreter;

#[test]
fn assert_success() {
    let code = "assert true!";
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn assert_fail() {
    let code = "assert false!";
    assert!(Interpreter::new_eval(code).is_err());
}

// TODO: implmenet
// #[test]
// fn print() {
//     interpreter_test_output("print 1!", "1\n");
// }

#[test]
fn get_typeof() {
    let code = r#"
var var type = typeof 1!
assert type === "number"!
var var type = typeof true!
assert type === "boolean"!
var var type = typeof false!
assert type === "boolean"!
var var type = typeof 1n!
assert type === "bigint"!
var var type = typeof \"\"!
assert type === "string"!
var var type = typeof undefined!
assert type === "undefined"!
var var type = typeof {}!
assert type === "object"!
var var type = typeof null!
assert type === "object"!
"#;
    // TODO
    // interpreter_test_output("print typeof Symbol()!", "symbol\n");
    // TODO
    // interpreter_test_output("print typeof []!", "object\n");
    Interpreter::new_eval(code).unwrap();
}
