use super::*;

#[test]
fn function_expression_minimum() {
    let body = " =>statement!";

    let (body, (args, statement)) = function_expression(Position::new(body)).unwrap();

    assert!(args.is_none());
    assert_eq!(statement.input, "statement");
    assert_eq!(statement.column, 4);
    assert_eq!(statement.line, 1);
    assert_eq!(statement.index, 3);

    assert_eq!(body.input, "!");
    assert_eq!(body.column, 13);
    assert_eq!(body.line, 1);
    assert_eq!(body.index, 12);
}
