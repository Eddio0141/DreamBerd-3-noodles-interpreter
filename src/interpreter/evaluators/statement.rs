use nom::Parser;

use crate::{
    interpreter::{evaluators::function::FunctionCall, runtime},
    parsers::types::Position,
    Interpreter,
};

use super::{parsers::AstParseResult, variable::VariableDecl};

pub enum Statement {
    FunctionCall(FunctionCall),
    VariableDecl(VariableDecl),
}

impl Statement {
    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        let mut function_call = FunctionCall::parse.map(|o| Statement::FunctionCall(o));

        // test for function call
        if let Ok(result) = function_call.parse(input) {
            return Ok(result);
        }

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

    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), runtime::error::Error> {
        match self {
            Statement::FunctionCall(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::VariableDecl(statement) => statement.eval(interpreter).map(|_| ()),
        }
    }
}
