use nom::{branch::*, combinator::*, sequence::*, Parser};

use crate::{
    interpreter::{
        evaluators::{function::FunctionCall, variable::VarSet},
        runtime,
    },
    parsers::{types::Position, *},
    Interpreter,
};

use super::{function::FunctionDef, parsers::AstParseResult, scope::*, variable::VariableDecl};

#[derive(Debug)]
pub enum Statement {
    FunctionCall(FunctionCall),
    FunctionDef(FunctionDef),
    VariableDecl(VariableDecl),
    VarSet(VarSet),
    Expression,
    ScopeStart(ScopeStart),
    ScopeEnd(ScopeEnd),
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
        let function_call = tuple((FunctionCall::parse, end_of_statement))
            .map(|(func, _)| Statement::FunctionCall(func));
        let function_def = FunctionDef::parse.map(Statement::FunctionDef);
        let variable_decl = VariableDecl::parse.map(Statement::VariableDecl);
        let var_set = VarSet::parse.map(Statement::VarSet);
        let scope_start = ScopeStart::parse.map(Statement::ScopeStart);
        let scope_end = ScopeEnd::parse.map(Statement::ScopeEnd);

        if let Ok((input, statement)) = alt((
            function_call,
            function_def,
            variable_decl,
            var_set,
            scope_start,
            scope_end,
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

            if let Ok((input_new, _)) = alt((ws1, chunk.map(|_| ())))(input) {
                input = input_new;
            }
        }
    }

    pub fn eval(&self, interpreter: &Interpreter, code: &str) -> Result<(), runtime::error::Error> {
        match self {
            Statement::FunctionCall(statement) => statement.eval(interpreter, code).map(|_| ()),
            Statement::FunctionDef(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::VariableDecl(statement) => statement.eval(interpreter, code).map(|_| ()),
            Statement::VarSet(statement) => statement.eval(interpreter, code).map(|_| ()),
            Statement::Expression => Ok(()),
            Statement::ScopeStart(statement) => statement.eval(interpreter).map(|_| ()),
            Statement::ScopeEnd(statement) => statement.eval(interpreter).map(|_| ()),
        }
    }
}
