use nom::combinator::peek;

use crate::{
    interpreter::evaluators::function::FunctionCall, parsers::types::Position, Interpreter,
};

use super::{parsers::EvalResult, variable::VariableDecl};

pub enum Statement<'a> {
    VariableDecl(VariableDecl<'a>),
}

impl<'a> Statement<'a> {
    pub fn parse(input: Position<&Interpreter>) -> EvalResult<'a, Self> {
        // test for function call
        if let Ok((_, next_chunk)) = peek(FunctionCall::parse)(input) {}

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

    pub fn eval(&self) {
        todo!()
    }
}
