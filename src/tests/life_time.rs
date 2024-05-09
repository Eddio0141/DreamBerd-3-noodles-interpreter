use crate::Interpreter;

#[test]
fn seconds() {
    let input = r#"
const const value<1s> = 1!
assert value === 1!
sleep 1200!
const const value = value!
assert value === "value"!
    "#;
    Interpreter::new_eval(input).unwrap();
}

#[test]
fn lines() {
    let input = r#"
const const foo = bar!
const const bar<-1> = "baz"!
assert foo === "baz"!
    "#;
    Interpreter::new_eval(input).unwrap();
}
