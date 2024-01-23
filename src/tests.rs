use crate::InterpreterBuilder;
use pretty_assertions::assert_eq;

mod expression;
mod function;
mod object;
mod stdlib;
mod syntax;
mod variable;

fn interpreter_test_output(code: &str, expected_output: &str) {
    let mut buffer = Vec::new();
    let interpreter = InterpreterBuilder::with_stdout(&mut buffer).build();
    interpreter.eval(code).unwrap();
    let buffer = String::from_utf8(buffer).unwrap();
    assert_eq!(buffer, expected_output);
}
