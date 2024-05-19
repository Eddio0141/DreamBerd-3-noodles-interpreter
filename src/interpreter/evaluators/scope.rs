use crate::{
    interpreter::runtime::{value::Value, Error},
    parsers::{ws, PosWithInfo},
    Interpreter,
};
use nom::{
    branch::alt, character::complete::char, combinator::eof, multi::many_till, sequence::tuple,
    Parser,
};

use super::{parsers::AstParseResult, statement::Statement};

#[derive(Debug)]
pub struct ScopeStart {
    line: usize,
}

impl ScopeStart {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let scope_start = char('{');
        let line = input.line;

        let (input, _) = scope_start(input)?;

        Ok((input, Self { line }))
    }

    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        interpreter.state.push_scope(self.line);
        Ok(Value::Undefined)
    }
}

#[derive(Debug)]
pub struct ScopeEnd {
    line: usize,
}

impl ScopeEnd {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        let scope_end = char('}');
        let line = input.line;

        let (input, _) = scope_end(input)?;

        Ok((input, Self { line }))
    }

    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        interpreter.state.pop_scope(self.line);
        Ok(Value::Undefined)
    }
}

// this parses the function body
// properly checks scope balance
pub fn scope(input: PosWithInfo) -> AstParseResult<()> {
    let scope_start = char('{');
    let (mut input, _) = scope_start(input)?;
    let scope_start = || tuple((ws, char('{'))).map(|_| Some(true));

    let scope_end = || tuple((ws, char('}'))).map(|_| Some(false));
    let mut statements_in_scope = many_till(
        Statement::parse,
        alt((scope_start(), scope_end(), eof.map(|_| None))),
    );

    let mut scope_track = 1usize;
    loop {
        if let Ok((i, (_, open_scope))) = statements_in_scope.parse(input) {
            input = i;

            if let Some(open_scope) = open_scope {
                if open_scope {
                    scope_track = scope_track.checked_add(1).expect("scope overflow");
                } else {
                    scope_track -= 1;
                    if scope_track == 0 {
                        return Ok((input, ()));
                    }
                }

                continue;
            }
        }

        // this basically parses the rest of the code as this function's body, and this is fine
        // TODO parse the function as implicit string if it doesn't end with a scope?
        return Ok((input, ()));
    }
}
