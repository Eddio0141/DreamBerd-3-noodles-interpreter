use nom::{
    branch::alt,
    combinator::{eof, value},
    multi::many_till,
    sequence::tuple,
    Parser,
};

use crate::{
    interpreter::{
        evaluators::{function::FunctionCall, variable::VarSet},
        runtime,
    },
    parsers::{types::Position, *},
    Interpreter,
};

use super::{function::FunctionDef, parsers::AstParseResult, variable::VariableDecl};

pub enum Statement {
    FunctionCall(FunctionCall),
    FunctionDef(FunctionDef),
    VariableDecl(VariableDecl),
    VarSet(VarSet),
    Expression,
}

impl Statement {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        let (input, _) = ws(input).unwrap();

        if input.input.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        // this needs to be done here since functions can be recursive
        let function_call = tuple((FunctionCall::parse, end_of_statement));

        if let Ok((input, statement)) = alt((
            function_call.map(|(func, _)| Statement::FunctionCall(func)),
            FunctionDef::parse.map(Statement::FunctionDef),
            VariableDecl::parse.map(Statement::VariableDecl),
            VarSet::parse.map(Statement::VarSet),
        ))(input)
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
            Statement::FunctionDef(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::VariableDecl(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::VarSet(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::Expression => Ok(()),
        }
    }
}
