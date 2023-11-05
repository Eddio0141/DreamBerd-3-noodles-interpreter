use crate::InterpreterBuilder;

mod expression;
mod stdlib;
mod syntax;
mod variable;

fn interpreter_test_output(code: &str, expected_output: &str) {
    let mut buffer = Vec::new();
    let interpreter = InterpreterBuilder::with_stdout(&mut buffer).build();
    interpreter.eval(code).unwrap();
    assert_eq!(buffer, expected_output.as_bytes());
}
