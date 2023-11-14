//! Contains variable related structures

use nom::bytes::complete::tag;
use nom::combinator::peek;
use nom::sequence::Tuple;
use nom::IResult;

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::expression::Expression;
use super::function::FunctionCall;
use super::parsers::*;
use super::{ParserInput, Statement};

#[derive(Debug, Clone)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    expression: Expression,
}

impl VariableDecl {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.add_var(&self.name, value);

        Ok(())
    }

    pub fn parse(mut input: ParserInput) -> IResult<ParserInput, Statement> {
        let funcs = input.static_analysis.current_funcs();

        if let Some((_, func)) = funcs.get_key_value("var") {
            if func.arg_count != 0 {
                // not a variable declaration
                return FunctionCall::parse(input)
                    .map(|(left, func)| (left, Statement::FunctionCall(func)));
            }
        }

        // not a function call, is a declaration
        let var = tag("var");

        // var ~ ws+ ~ var ~ ws+ ~ ident ~ ws* ~ "=" ~ ws* ~ expr ~ ws* ~ "!"
        // note: ident can be chained with =, but ident itself can be =
        let (input, (_, _, _, _, identifier, _, _, _, expression, _, term)) = (
            var,
            ws1,
            var,
            ws1,
            identifier_optional_term('='),
            ws,
            equals::<_, nom::error::Error<ParserInput>>,
            ws,
            Expression::parse,
            ws,
            term,
        )
            .parse(input)?;

        let decl = Self {
            expression: expression.into(),
            name: identifier.to_string(),
        };

        // input.code = code;

        Ok((input, Statement::VariableDecl(decl)))
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
        interpreter.state.set_var(&self.name, value);
        Ok(())
    }

    pub fn parse(mut input: ParserInput) -> IResult<ParserInput, Statement> {
        let funcs = input.static_analysis.current_funcs();

        let identifier = identifier_optional_term('=');

        let (input, identifier_peek) = peek(identifier)(input)?;

        if let Some((_, func)) = funcs.get_key_value(identifier_peek) {
            if func.arg_count != 0 {
                // not a variable declaration
                return FunctionCall::parse(input)
                    .map(|(input, func)| (input, Statement::FunctionCall(func)));
            }
        }

        // ident ~ ws* ~ "=" ~ ws* ~ expr ~ ws* ~ !
        let (input, (_, _, _, _, expression, _, _)) = (
            identifier,
            ws,
            equals::<_, nom::error::Error<ParserInput>>,
            ws,
            Expression::parse,
            ws,
            term,
        )
            .parse(input)?;

        let var_set = Self {
            name: identifier_peek.to_string(),
            expression: expression.into(),
        };

        Ok((input, Statement::VarSet(var_set)))
    }
}
