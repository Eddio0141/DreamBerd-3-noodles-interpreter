use std::io::Cursor;

use crate::{interpreter::InterpreterBuilder, Interpreter};

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
    let code = "print 1!";
    let mut buffer = Cursor::new(Vec::new());
    let interpreter = InterpreterBuilder::with_stdout(&mut buffer).build();
    interpreter.eval(code).unwrap();
    assert_eq!(buffer.get_ref(), b"1\n");
}
