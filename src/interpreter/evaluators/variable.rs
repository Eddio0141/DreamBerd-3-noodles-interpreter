//! Contains variable related structures

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::multi::many1;
use nom::sequence::Tuple;
use nom::Parser;

use crate::parsers::types::Position;
use crate::parsers::{end_of_statement, identifier, ws, LifeTime};

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::expression::{AtomPostfix, Expression};
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

    pub fn parse<'a, 'b>(input: Position<'a, Interpreter<'b>>) -> AstParseResult<'a, 'b, Self> {
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
    postfix: Vec<AtomPostfix>,
    expression: Expression,
    line: usize,
}

impl VarSet {
    pub fn eval(&self, args: EvalArgs) -> Result<(), Error> {
        let interpreter = args.1.extra;
        let value = self.expression.eval(args)?;
        interpreter.state.set_var(
            &self.name,
            args,
            &self.postfix,
            value.0.into_owned(),
            self.line,
        )?;
        Ok(())
    }

    pub fn parse<'a, 'b>(
        input_orig: Position<'a, Interpreter<'b>>,
    ) -> AstParseResult<'a, 'b, Self> {
        // ident ws* "=" ws* expr ws* !
        let eq = char('=');
        let mut identifier_full = identifier(LifeTime::parse);
        let (mut input, mut var_identifier) = identifier_full(input_orig)?;

        let mut postfix = None;
        if input_orig
            .extra
            .state
            .get_var(var_identifier.input)
            .is_none()
        {
            // try with postfix
            if let Ok((input_, identifier_postfix)) = identifier(alt((
                AtomPostfix::parse.map(|_| ()),
                LifeTime::parse.map(|_| ()),
            )))(input_orig)
            {
                // has postfix
                var_identifier = identifier_postfix;
                let (input_, postfix_) = many1(AtomPostfix::parse)(input_).unwrap();
                input = input_;
                postfix = Some(postfix_);
            }
        }
        let (input, (_, _, _, expression, _)) =
            (ws, eq, ws, Expression::parse, end_of_statement).parse(input)?;

        let decl = Self {
            expression,
            name: var_identifier.input.to_string(),
            line: var_identifier.line,
            postfix: postfix.unwrap_or_default(),
        };

        Ok((input, decl))
    }
}
