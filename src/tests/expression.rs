use crate::Interpreter;

use super::interpreter_test_output;

#[test]
fn int_comparisons() {
    let code = r#"
assert(1 === 1)!
assert(1 ;== 2)!
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
assert(3 === 2)!
"#;
    // assert(4 === 1)!
    // assert(3 ;== 4)!
    // assert(4 < 3)!
    // assert(3 <= 2)!
    // assert(3 > 4)!
    // assert(4 >= 1)!
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparisons_chain() {
    // the first comparison converts it into a boolean
    let code = r#"
assert(1 === 1 === true)!
assert(1 ;== 2 === true)!
assert(1 > 2 === false)!
assert(1 <= 1 === true)!
assert(2 > 1 === true)!
assert(1 >= 2 === false)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparison_order() {
    let code = r#"
assert(false || true&&false  === false)!
assert(false === 1===2)!
assert(true === 1;==2)!
    "#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comparison_no_ws() {
    let code = r#"
assert(1===1===true)!
assert(1;==2===true)!
assert(1>2===false)!
assert(1<=1===true)!
assert(2>1===true)!
assert(1>=2===false)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn math_expr() {
    let code = r#"
assert(1+1===2)!
assert(1-1===0)!
assert(2*3===6)!
assert(6/2===3)!
assert(6%2===0)!
assert(6^2===36)!
assert(-6*-2===6*2)!
assert(--6===6)!
assert(6--6===12)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn math_expr_unary() {
    let code = r#"
assert(-1 === -1)!
assert(- 1+2 ===- 1+2)!
assert(-6*- 2+3 ===6*5)!
assert(- 6+2 ===-8)!
assert(-  6+2+ 3  ===-11)!
assert(- 6+2+ 3 ===-5)!
assert(-6+2===-4)!
assert(- ;0 ===-1)!
assert(- --1+2 ===-3)!
assert(- 1+2 +- 3+4 ===-10)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn math_expr_unary2() {
    let code = r#"
print - -1!
print - - -1!
print - - - -1!
"#;

    interpreter_test_output(code, "1\n-1\n1\n");
}

#[test]
fn math_expr_order() {
    // 1 + 1 * 2 = 3
    // (1 - 1) * 2 = 0
    // 2 * 3 + 1 = 7
    // 2 * (3 + 1) = 8
    // 2 * (3 + (4 * 5)) + 6 = 52
    // 2 * -(-3+4) - -5 + - -6 = 9
    // 2 * --3 + 4 = 10
    // 2 * -(3 + 4) = -14
    let code = r#"
assert(1+1*2===3)!
assert(1-1 * 2  === 0)!
assert(2*3+1===7)!
assert(2 *3+1  ===  8)!
assert(2 * 3+ 4*5 + 6  ===52)!
assert(2 * - -3+4 - -5 + - -6    === 9)!
assert(2 * - - 3 + 4    ===    10)!
assert(2 * - 3+4    ===    -14)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn divide_by_zero() {
    let code = "assert(1/0 === undefined)!";
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn loose_comparison() {
    // TODO object comparison
    // TODO string comparison
    // TODO float comparison
    // TODO bigint comparison
    // TODO symbol comparison
    // TODO object to primitive comparison, same type
    // TODO bool to string comparison
    // TODO number to string comparison
    // TODO string to number comparison
    // TODO number to bigint comparison
    // TODO bigint to number comparison
    // TODO string to bigint comparison
    // TODO bigint to string comparison
    let code = r#"
assert(15 == 15)!
assert(true == true)!
assert(false == false)!
assert(null == null)!
assert(undefined == undefined)!
assert(null ;= undefined)!
assert(true == 1)!
assert(false == 0)!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comp_less_than() {
    // TODO object comparison [@@toPrimitive](), valueOf(), toString()
    // TODO string comparison
    // TODO compare with bigint
    // TODO compare with null
    // TODO symbol comparison
    // TODO float comparison
    let code = r#"
assert true < 2!
assert false < 1!
assert undefined < 1 === false!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comp_strict() {
    // TODO null and undefined comparison
    // TODO test NaN comparison
    // TODO implicit string compare with string
    let code = r#"
assert null === null!
assert(1 === 1)!
assert(true ;== 1)!
assert(false ;== 0)!
assert 1.5 === 1.5!
assert(undefined === undefined)!
assert "hello" === "hello"!
"#;
    Interpreter::new_eval(code).unwrap();
}

#[test]
fn comp_strict_neg() {
    // TODO null and null comparison
    // TODO null and undefined comparison
    // TODO test NaN comparison
    // TODO float comparison
    // TODO string comparison
    let code = r#"
assert(1 ;== 1 === false)!
assert(true ;== 1)!
assert(false ;== 0)!
assert(undefined ;== undefined === false)!
"#;
    Interpreter::new_eval(code).unwrap();
}
