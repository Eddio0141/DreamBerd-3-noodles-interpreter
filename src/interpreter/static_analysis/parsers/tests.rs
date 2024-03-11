use super::*;

#[test]
fn function_expression_minimum() {
    let body = " =>statement!";

    function_expression(Position::new(body)).unwrap();
}

#[test]
fn function_expression_arg_minimum() {
    let body = "arg =>statement!";

    function_expression(Position::new(body)).unwrap();
}

#[test]
fn function_expression_args_minimum() {
    let body = "arg,arg2,arg3,=>,arg5 =>statement!";

    function_expression(Position::new(body)).unwrap();
}

#[test]
fn till_term_statement() {
    let code = "statement!!statement2!";
    let (input, _) = till_term(Position::new(code)).unwrap();
    assert_eq!(input.input, "!!statement2!");
    let input = Position::new(&input.input[2..]);

    let (input, _) = till_term(input).unwrap();
    assert_eq!(input.input, "!");
}

#[test]
fn till_term_with_strings() {
    let code = "statement!foo\"statement2\"bar!";
    let (input, _) = till_term(Position::new(code)).unwrap();
    assert_eq!(input.input, "!foo\"statement2\"bar!");
    let input = Position::new(&input.input[1..]);

    let (input, _) = till_term(input).unwrap();
    assert_eq!(input.input, "!");
}
