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
