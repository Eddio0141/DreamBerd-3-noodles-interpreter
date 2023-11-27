use nom::{
    branch::alt,
    combinator::{eof, value},
    multi::many_till,
    sequence::terminated,
    Parser,
};

use crate::{
    interpreter::{evaluators::function::FunctionCall, runtime},
    parsers::{chunk, end_of_statement, types::Position, ws},
    Interpreter,
};

use super::{parsers::AstParseResult, variable::VariableDecl};

pub enum Statement {
    FunctionCall(FunctionCall),
    VariableDecl(VariableDecl),
    Expression,
}

impl Statement {
    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        let (input, _) = ws(input).unwrap();

        if input.input.is_empty() {
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        if let Ok((input, statement)) = terminated(
            alt((
                FunctionCall::parse.map(Statement::FunctionCall),
                VariableDecl::parse.map(Statement::VariableDecl),
            )),
            end_of_statement,
        )(input)
        {
            return Ok((input, statement));
        }

        todo!("{}", input.input);

        // TODO rewrite test to ensure type isn't implicit string
        // last resort, pass it as an implicit string
        many_till(alt((ws, chunk)), alt((value((), eof), end_of_statement)))
            .map(|_| Self::Expression)
            .parse(input)

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
            Statement::Expression => Ok(()),
        }
    }
}
