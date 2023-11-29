use crate::{
    interpreter::runtime::{value::Value, Error},
    parsers::types::Position,
    Interpreter,
};
use nom::character::complete::char;

use super::parsers::AstParseResult;

pub struct ScopeStart {
    line: usize,
}

impl ScopeStart {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
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

pub struct ScopeEnd {
    line: usize,
}

impl ScopeEnd {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
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
