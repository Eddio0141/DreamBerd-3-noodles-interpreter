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
function foo arg => arg!
var var output = foo 1!
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
fn declare_function_arg_scope() {
    let code = r#"
function foo arg => {
    return arg!
}
var var output = foo 1!
assert(output === 1)!
var var arg_type = typeof(arg)!
assert(arg_type === "string")!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_function_args_scope() {
    let code = r#"
function add left, right => {
    return left + right!
}
var var output = add 1, 2!
assert(output === 3)!
var var left_type = typeof(left)!
assert(left_type === "string")!
var var right_type = typeof(right)!
assert(right_type === "string")!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn function_nested_scopes_return() {
    let code = r#"
function add left, right => {
    {
        var var foo = 1!
        {{
            return left + right!
        }}
    }
}
add 1, 2!
var var left_type = typeof(left)!
assert(left_type === "string")!
var var right_type = typeof(right)!
assert(right_type === "string")!
var var foo_type = typeof(foo)!
assert(foo_type === "string")!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn empty_function() {
    let code = r#"
function foo => {}
foo!
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

#[test]
fn scope_close_mass() {
    let code = r#"
var var a = 1!
{{{{
    assert(a === 1)!
    "#;

    Interpreter::new_eval(code).unwrap();
}
