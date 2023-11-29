use super::*;

#[test]
fn function_expression_minimum() {
    let body = " =>statement!";

    let (body, (args, statement)) = function_expression(Position::new(body)).unwrap();

    assert!(args.is_empty(), "args is not empty: {:?}", args);
    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 4);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 3);

    assert!(body.input.is_empty());
    assert_eq!(body.column, 14);
    assert_eq!(body.line, 1);
    assert_eq!(body.index, 13);
}

#[test]
fn function_expression_arg_minimum() {
    let body = "arg =>statement!";

    let (body, (args, statement)) = function_expression(Position::new(body)).unwrap();

    assert_eq!(args.len(), 1);
    let arg = &args[0];
    assert_eq!(arg.input, "arg");
    assert_eq!(arg.line, 1);
    assert_eq!(arg.column, 1);
    assert_eq!(arg.index, 0);

    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 7);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 6);

    assert!(body.input.is_empty());
    assert_eq!(body.column, 17);
    assert_eq!(body.line, 1);
    assert_eq!(body.index, 16);
}

#[test]
fn function_expression_args_minimum() {
    let body = "arg,arg2,arg3,=>,arg5 =>statement!";

    let (body, (args, statement)) = function_expression(Position::new(body)).unwrap();

    assert_eq!(args.len(), 5);
    assert_eq!(args[0].input, "arg");
    assert_eq!(args[1].input, "arg2");
    assert_eq!(args[2].input, "arg3");
    assert_eq!(args[3].input, "=>");
    let arg5 = &args[4];
    assert_eq!(arg5.input, "arg5");
    assert_eq!(arg5.line, 1);
    assert_eq!(arg5.column, 18);
    assert_eq!(arg5.index, 17);

    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 25);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 24);

    assert!(body.input.is_empty());
    assert_eq!(body.column, 35);
    assert_eq!(body.line, 1);
    assert_eq!(body.index, 34);
}

#[test]
fn till_term_statement() {
    let code = "statement!!statement2!";
    let (input, statement) = till_term(Position::new(code)).unwrap();
    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 1);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 0);
    assert_eq!(input.input, "statement2!");

    let (input, statement) = till_term(input).unwrap();
    assert_eq!(statement.input, "statement2");
    assert_eq!(statement.column, 12);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 11);
    assert!(input.input.is_empty());
}

#[test]
fn till_term_with_strings() {
    let code = "statement!foo\"statement2\"bar!";
    let (input, statement) = till_term(Position::new(code)).unwrap();
    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 1);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 0);
    assert_eq!(input.input, "foo\"statement2\"bar!");

    let (input, statement) = till_term(input).unwrap();
    assert_eq!(statement.input, "foo\"statement2\"bar");
    assert_eq!(statement.column, 11);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 10);
    assert!(input.input.is_empty());
}
