use crate::Interpreter;

#[test]
fn multiple_end() {
    let code = "var var a = 1!!!!!!!";
    Interpreter::new_eval(code).unwrap();
}
