use crate::Interpreter;

#[test]
fn declare_function_no_args() {
    let code = r#"
function test => 1!
var var output = test!
assert(output === 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_function_arg() {
    let code = r#"
function add left => left!
var var output = add 1!
assert(output === 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_function_args() {
    let code = r#"
function add left, right => left + right!
var var output = add 1, 2!
assert(output === 3)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn scope_test() {
    let code = r#"
var var a = 1!
{
    var var a = 2!
    assert(a === 2)!
}
assert(a === 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn scope_test2() {
    let code = r#"
var var a = 1!
{
    var var a = 2!
    assert(a === 2)!
    {
        var var a = 3!
        assert(a === 3)!
    }
    assert(a === 2)!
}
assert(a === 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}
