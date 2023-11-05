use crate::Interpreter;

use super::interpreter_test_output;

#[test]
fn multiple_end() {
    let code = "var var a = 1!!!!!!!";
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn hello_world() {
    interpreter_test_output("print \"Hello, World!\"!", "Hello, World!\n");
}

#[test]
fn string_no_quotes_no_space() {
    interpreter_test_output("print hello_world!", "hello_world\n");
}

// #[test]
// fn string_no_quotes_spaces() {
//     interpreter_test_output("print hello world!", "hello world\n");
// }

// #[test]
// fn string_many_quotes() {
//     let code = r#"
// print """"""hello world""""""!
// print '''''hello world'''''!
// "#;
//     interpreter_test_output(code, "hello world\nhello world\n");
// }

// #[test]
// fn string_mixed_quotes() {
//     let code = r#"
// print ""''""hello world"""''""!
// print ''""''hello world'''""''!
// "#;
//     interpreter_test_output(code, "hello world\"\nhello world\'\n");
// }

// #[test]
// fn string_single_quote() {
//     let code = r#"
// print 'hello world!
// print hello world"!
// "#;
//     interpreter_test_output(code, "\'hello world!\nhello world\"\n");
// }

#[test]
fn string_escape_chars() {
    let code = r#"print "\' \" \\ \n"!"#;
    interpreter_test_output(code, "\' \" \\ \n\n");
}
