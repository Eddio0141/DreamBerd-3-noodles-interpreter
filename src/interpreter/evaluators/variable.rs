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
use super::EvalArgs;

#[derive(Debug, Clone)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    expression: Expression,
    line: usize,
}

impl VariableDecl {
    pub fn eval(&self, args: EvalArgs) -> Result<(), Error> {
        let interpreter = args.1.extra;
        let value = self.expression.eval(args)?;
        interpreter
            .state
            .add_var(&self.name, value.0.into_owned(), self.line);

        Ok(())
    }

    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        let var = || tag("var");
        let eq = char('=');
        let identifier = identifier(LifeTime::parse);
        // var ws+ var ws+ identifier life_time? ws* "=" ws* expr
        let (input, (start, _, _, _, identifier, _, _, _, _, expression, _)) = (
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
            line: start.line,
        };

        Ok((input, decl))
    }
}

#[derive(Debug, Clone)]
pub struct VarSet {
    name: String,
    expression: Expression,
    line: usize,
}

impl VarSet {
    pub fn eval(&self, args: EvalArgs) -> Result<(), Error> {
        let interpreter = args.1.extra;
        let value = self.expression.eval(args)?;
        interpreter
            .state
            .set_var(&self.name, value.0.into_owned(), self.line);
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
            line: identifier.line,
        };

        Ok((input, decl))
    }
}
