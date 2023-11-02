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

#[test]
fn math_expr() {
    let code = r#"
assert(1+1==2)!
assert(1-1==0)!
assert(2*3==6)!
assert(6/2==3)!
assert(6%2==0)!
assert(6**2==36)!
assert(-6*-2==6*2)!
assert(--6==6)!
assert(6--6==12)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn math_expr_order() {
    // 1 + 1 * 2 = 3
    // (1 - 1) * 2 = 0
    // 2 * 3 + 1 = 7
    // 2 * (3 + 1) = 8
    // 2 * (3 + (4 * 5)) + 6 = 52
    // 2 * -(-3 + 4) - -5 + -(-6) = 9
    // 2 * -(-(3 + 4)) = 14
    // (2 * -(3 + 4)) = -14
    let code = r#"
assert(1+1*2==3)!
assert(1-1 * 2  == 0)!
assert(2*3+1==7)!
assert(2 *3+1  ==  8)!
assert(2 * 3+ 4*5 + 6  ==52)!
assert(2 * - -3+4 - -5 + - -6    == 9)!
assert(2 * - - 3 + 4    ==    14)!
assert(2 * - 3 + 4    ==    -14)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn divide_by_zero() {
    let code = "assert(1/0 == undefined)!";
    Interpreter::new_eval(code).unwrap();
}

// #[test]
// fn undefined_comparisons() {
//     let code = r#"
// assert(undefined == undefined)!"#;
//     Interpreter::new_eval(code).unwrap();
// }
