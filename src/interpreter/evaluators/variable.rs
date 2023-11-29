//! Contains variable related structures

use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::sequence::Tuple;

use crate::parsers::types::Position;
use crate::parsers::{end_of_statement, identifier, ws, LifeTime};

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::expression::Expression;
use super::parsers::AstParseResult;

#[derive(Debug, Clone)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    expression: Expression,
}

impl VariableDecl {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.add_var(&self.name, value.0.into_owned());

        Ok(())
    }

    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        let var = || tag("var");
        let eq = char('=');
        let identifier = identifier(LifeTime::parse);
        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr
        let (input, (_, _, _, _, identifier, _, _, _, _, expression, _)) = (
            var(),
            ws,
            var(),
            ws,
            identifier,
            opt(LifeTime::parse),
            ws,
            eq,
            ws,
            Expression::parse,
            end_of_statement,
        )
            .parse(input)?;

        let decl = Self {
            expression,
            name: identifier.input.to_string(),
        };

        Ok((input, decl))
    }
}

#[derive(Debug, Clone)]
pub struct VarSet {
    name: String,
    expression: Expression,
}

impl VarSet {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.set_var(&self.name, value.0.into_owned());
        Ok(())
    }

    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        // ident ws* "=" ws* expr ws* !
        let eq = char('=');
        let identifier = identifier(LifeTime::parse);
        let (input, (identifier, _, _, _, expression, _)) =
            (identifier, ws, eq, ws, Expression::parse, end_of_statement).parse(input)?;

        let decl = Self {
            expression,
            name: identifier.input.to_string(),
        };

        Ok((input, decl))
    }
}
