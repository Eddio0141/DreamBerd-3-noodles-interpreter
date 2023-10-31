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
