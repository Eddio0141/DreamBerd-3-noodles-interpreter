use crate::Interpreter;

#[test]
fn int_comparisons() {
    let code = r#"
assert(1 == 1)!
assert(1 ;= 2)!
assert(1 < 2)!
assert(1 <= 1)!
assert(2 > 1)!
assert(1 >= 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn int_comparisons_variable() {
    let code = r#"
var var 3 = 2!
var var 4 = 1!
assert(3 == 2)!
assert(4 == 1)!
assert(3 ;= 4)!
assert(4 < 3)!
assert(3 <= 2)!
assert(3 > 4)!
assert(4 >= 1)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparisons_chain() {
    // the first comparison converts it into a boolean
    let code = r#"
assert(1 == 1 == true)!
assert(1 ;= 2 == true)!
assert(1 > 2 == false)!
assert(1 <= 1 == true)!
assert(2 > 1 == true)!
assert(1 >= 2 == false)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparison_order() {
    let code = r#"
    assert(false || true&&false  == false)!
    "#;
    // assert(false == 1==2)!
    // assert(true == 1;=2)!
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparison_no_ws() {
    let code = r#"
assert(1==1==true)!
assert(1;=2==true)!
assert(1>2==false)!
assert(1<=1==true)!
assert(2>1==true)!
assert(1>=2==false)!
"#;
    Interpreter::new_eval(code).unwrap();
}

// #[test]
// fn undefined_comparisons() {
//     let code = r#"
// assert(undefined == undefined)!"#;
//     Interpreter::new_eval(code).unwrap();
// }
