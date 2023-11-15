use nom::IResult;

use crate::Interpreter;

pub fn eval_statement<'a>(code: &'a str, interpreter: &Interpreter) -> IResult<&'a str, &'a str> {
    todo!()

    // alt((
    //     VariableDecl::parse,
    //     VarSet::parse,
    //     Ast::parse_scope,
    //     // those are fallback parsers
    //     map(FunctionCall::parse, |func| Statement::FunctionCall(func)),
    //     Expression::parse,
    // ))(input)
}
