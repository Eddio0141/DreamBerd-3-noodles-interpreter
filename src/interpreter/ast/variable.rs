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
                return FunctionCall::parse(input).map(|(_, func)| Statement::FunctionCall(func));
            }
        }

        // not a function call, is a declaration
        let var = tag("var");
        let ws1 = ws1();
        let ws = ws();
        let identifier = identifier_optional_term('=');
        let eq = equals();
        let term = term();

        // var ~ ws+ ~ var ~ ws+ ~ ident ~ ws* ~ "=" ~ ws* ~ expr ~ ws* ~ "!"
        // note: ident can be chained with =, but ident itself can be =
        let code = input.code;
        let (code, (_, _, _, _, identifier, _, _, _, expression, _, term)) = (
            var,
            ws1,
            var,
            ws1,
            identifier,
            ws,
            eq,
            ws,
            Expression::parse,
            ws,
            term,
        )
            .parse(code)?;

        let decl = Self {
            expression,
            name: identifier.to_string(),
        };

        input.code = code;

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

        let code = input.code;
        let (code, identifier) = peek(identifier)(code)?;

        if let Some((_, func)) = funcs.get_key_value(identifier) {
            if func.arg_count != 0 {
                // not a variable declaration
                return FunctionCall::parse(input).map(|(_, func)| Statement::FunctionCall(func));
            }
        }

        // ident ~ ws* ~ "=" ~ ws* ~ expr ~ ws* ~ !
        let (code, (_, _, _, _, expression, _, term)) =
            (identifier, ws, equals, ws, Expression::parse, ws, term).parse(code)?;

        let var_set = Self {
            name: identifier.to_string(),
            expression,
        };

        input.code = code;

        Ok((input, Statement::VarSet(var_set)))
    }
}
