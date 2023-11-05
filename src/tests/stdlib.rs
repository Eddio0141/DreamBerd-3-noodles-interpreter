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
