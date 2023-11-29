use crate::Interpreter;

use super::interpreter_test_output;

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

#[test]
fn print() {
    interpreter_test_output("print 1!", "1\n");
}

#[test]
fn get_typeof() {
    interpreter_test_output("print typeof 1!", "number\n");
    interpreter_test_output("print typeof true!", "boolean\n");
    interpreter_test_output("print typeof false!", "boolean\n");
    interpreter_test_output("print typeof 1n!", "bigint\n");
    interpreter_test_output("print typeof \"\"!", "string\n");
    interpreter_test_output("print typeof undefined!", "undefined\n");
    // TODO
    // interpreter_test_output("print typeof Symbol()!", "symbol\n");
    // TODO
    // interpreter_test_output("print typeof {}!", "object\n");
    // TODO
    // interpreter_test_output("print typeof []!", "object\n");
    interpreter_test_output("print typeof null!", "object\n");
}
