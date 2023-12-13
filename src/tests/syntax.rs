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

#[test]
fn string_no_quotes_spaces() {
    interpreter_test_output("print hello world!", "hello world\n");
}

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

#[test]
fn function_missing_order() {
    let code = r#"union a() => 1!
fn b() => 2!
n c() => 3!
fnto d() => 4!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comment_line() {
    let code = r#"
// This is a comment
print "Hello, World!"! // another comment
var var foo = // comment
    1!
print foo!
var var foo // comment
    = 2!
print foo!
"#;
    interpreter_test_output(code, "Hello, World!\n1\n2\n");
}

#[test]
fn comment_block() {
    let code = r#"
/* This is a comment
    that spans multiple lines */
print "Hello, World!"!
var var foo = /* comment */ 1!
print foo!
var var foo /* comment
    that spans multiple lines */ = 2!
print foo!
"#;
    interpreter_test_output(code, "Hello, World!\n1\n2\n");
}
