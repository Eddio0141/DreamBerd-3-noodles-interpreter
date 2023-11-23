use nom::{sequence::tuple, Parser};

use crate::{
    interpreter::{evaluators::function::FunctionCall, runtime},
    parsers::{end_of_statement, types::Position, ws},
    Interpreter,
};

use super::{parsers::AstParseResult, variable::VariableDecl};

pub enum Statement {
    FunctionCall(FunctionCall),
    VariableDecl(VariableDecl),
}

impl Statement {
    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        let function_call = FunctionCall::parse.map(|o| Statement::FunctionCall(o));

        let (input, _) = ws(input).unwrap();

        if input.input.is_empty() {
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        // test for function call
        if let Ok((input, (statement, _))) = tuple((function_call, end_of_statement)).parse(input) {
            return Ok((input, statement));
        }

        todo!("{:#}", input.input);

        // alt((
        //     VariableDecl::parse,
        //     VarSet::parse,
        //     Ast::parse_scope,
        //     // those are fallback parsers
        //     map(FunctionCall::parse, |func| Statement::FunctionCall(func)),
        //     Expression::parse,
        // ))(input)
    }

    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), runtime::error::Error> {
        match self {
            Statement::FunctionCall(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::VariableDecl(statement) => statement.eval(interpreter).map(|_| ()),
        }
    }
}
