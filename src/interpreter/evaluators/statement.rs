use crate::{
    parsers::types::{PosResult, Position},
    Interpreter,
};

use super::variable::VariableDecl;

pub enum Statement<'a> {
    VariableDecl(VariableDecl<'a>),
}

impl<'a> Statement<'a> {
    pub fn parse(input: Position, interpreter: &Interpreter) -> PosResult<'a, Self> {
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
