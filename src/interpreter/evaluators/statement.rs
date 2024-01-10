use nom::{branch::*, combinator::*, sequence::*, Parser};

use crate::{
    interpreter::{
        evaluators::{function::FunctionCall, variable::VarSet},
        runtime::{self, value::Value},
    },
    parsers::{types::Position, *},
    Interpreter,
};

use super::{
    function::{FunctionDef, Return},
    parsers::AstParseResult,
    scope::*,
    variable::VariableDecl,
    EvalArgs,
};

#[derive(Debug)]
pub enum Statement {
    FunctionCall(FunctionCall),
    FunctionDef(FunctionDef),
    VariableDecl(VariableDecl),
    VarSet(VarSet),
    Expression,
    ScopeStart(ScopeStart),
    ScopeEnd(ScopeEnd),
    Return(Return),
}

impl Statement {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        let (mut input, _) = ws::<_, ()>(input).unwrap();

        if input.input.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        // this needs to be done here since functions can be recursive
        let function_call = tuple((FunctionCall::parse_as_func, end_of_statement))
            .map(|(func, _)| Statement::FunctionCall(func));
        let function_def = FunctionDef::parse.map(Statement::FunctionDef);
        let variable_decl = VariableDecl::parse.map(Statement::VariableDecl);
        let var_set = VarSet::parse.map(Statement::VarSet);
        let scope_start = ScopeStart::parse.map(Statement::ScopeStart);
        let scope_end = ScopeEnd::parse.map(Statement::ScopeEnd);
        let ret = Return::parse.map(Statement::Return);

        if let Ok((input, statement)) = alt((
            function_call,
            function_def,
            variable_decl,
            var_set,
            scope_start,
            scope_end,
            ret,
        ))(input)
        {
            return Ok((input, statement));
        }

        // TODO rewrite test to ensure type isn't implicit string
        // last resort, pass it as an implicit string
        loop {
            if let Ok((input, _)) = alt((value((), eof::<_, ()>), end_of_statement))(input) {
                return Ok((input, Self::Expression));
            }

            if let Ok((input_new, _)) = alt((ws1, terminated_chunk.map(|_| ())))(input) {
                input = input_new;
            }
        }
    }

    pub fn eval(&self, args: EvalArgs) -> Result<Option<Value>, runtime::error::Error> {
        let interpreter = args.1.extra;
        match self {
            Statement::FunctionCall(statement) => statement.eval(args).map(|_| None),
            Statement::FunctionDef(statement) => statement.eval(interpreter).map(|_| None),
            Statement::VariableDecl(statement) => statement.eval(args).map(|_| None),
            Statement::VarSet(statement) => statement.eval(args).map(|_| None),
            Statement::Expression => Ok(None),
            Statement::ScopeStart(statement) => statement.eval(interpreter).map(|_| None),
            Statement::ScopeEnd(statement) => statement.eval(interpreter).map(|_| None),
            Statement::Return(statement) => statement.eval(args).map(Some),
        }
    }
}
