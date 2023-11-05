use crate::Interpreter;

#[test]
fn declare() {
    let code = r#"var var a = 1!
assert(a == 1)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_int() {
    let code = r#"var  var 1 = 2!
assert(1 == 2)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_var() {
    let code = r#"var var var = 1!
assert(var == 1)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_eq_symbol() {
    let code = r#"var var = = 1!
assert(= == 1)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_emoji() {
    let code = r#"var var 😀 = 1!
assert(😀 == 1)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_assert() {
    let code = r#"var var assert = 1!
assert(assert == 1)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn declare_spaces_swap() {
    let code = r#"var(var(a(=(1!
assert(a(==(1(!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn re_assign() {
    let code = r#"var var a = 1!
a = 2 * 3!
assert(a == 6)!"#;

    Interpreter::new_eval(code).unwrap();
}

#[test]
fn assign_non_existant() {
    let code = r#"a = 1!
assert(a == 1)!"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn name_string() {
    let code = r#"var var "a" = 1!
assert("a" === 1)!
assert("b" ;== 1)!
var var "b" = 1!
assert("b" === 1)!
assert("a" === "b")!
"#;
    Interpreter::new_eval(code).unwrap();
}
