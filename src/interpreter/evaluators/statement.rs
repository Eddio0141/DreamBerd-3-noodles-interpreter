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
    ImplicitString(Value),
    ScopeStart(ScopeStart),
    ScopeEnd(ScopeEnd),
    Return(Return),
}

impl Statement {
    pub fn parse<'a, 'b>(input: Position<'a, Interpreter<'b>>) -> AstParseResult<'a, 'b, Self> {
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
        let mut implicit_string = String::new();

        loop {
            if let Ok((input, _)) = alt((value((), eof::<_, ()>), end_of_statement))(input) {
                return Ok((input, Self::ImplicitString(Value::String(implicit_string))));
            }

            if let Ok((input_new, chunk)) = alt((ws1_value, terminated_chunk_value))(input) {
                input = input_new;
                implicit_string.push_str(&chunk);
            }
        }
    }

    pub fn eval(&self, args: EvalArgs) -> Result<StatementReturn, runtime::error::Error> {
        let interpreter = args.1.extra;
        let value = match self {
            Statement::FunctionCall(statement) => statement.eval(args)?,
            Statement::FunctionDef(statement) => {
                return statement.eval(interpreter).map(|_| Default::default())
            }
            Statement::VariableDecl(statement) => {
                return statement.eval(args).map(|_| Default::default())
            }
            Statement::VarSet(statement) => {
                return statement.eval(args).map(|_| Default::default())
            }
            Statement::ImplicitString(value) => value.to_owned(),
            Statement::ScopeStart(statement) => statement.eval(interpreter)?,
            Statement::ScopeEnd(statement) => statement.eval(interpreter)?,
            Statement::Return(statement) => {
                return statement.eval(args).map(|return_value| StatementReturn {
                    value: None,
                    return_value,
                })
            }
        };

        Ok(StatementReturn {
            value: Some(value),
            return_value: None,
        })
    }
}

#[derive(Default)]
pub struct StatementReturn {
    /// Any value that is generated from the statement
    pub value: Option<Value>,
    /// For return statements, this is the value that is returned
    pub return_value: Option<Value>,
}
