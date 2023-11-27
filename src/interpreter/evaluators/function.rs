//! Contains function related structures

use nom::{
    bytes::complete::*, character::complete::*, combinator::*, error::ErrorKind, multi::*,
    sequence::*, *,
};

use crate::{
    interpreter::runtime::{
        error::Error,
        state::{Function, FunctionVariant},
        value::Value,
    },
    parsers::types::Position,
};
use crate::{parsers::*, Interpreter};

use super::expression::Expression;
use super::parsers::AstParseResult;

#[derive(Debug, Clone)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall {
    name: String,
    args: Vec<Expression>,
}

impl FunctionCall {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(arg.eval(interpreter)?);
        }

        interpreter.state.invoke_func(interpreter, &self.name, args)
    }

    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        // function call syntax
        // - `func_name!`
        // with args
        // - `func_name arg1, arg2!`

        let mut identifier = identifier(fail::<_, Position<&Interpreter>, _>);

        let (input, identifier) = identifier(input)?;
        let identifier = identifier.into();

        // does the function exist
        let Some(func) = input.extra.state.get_func_info(identifier) else {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                ErrorKind::Fail,
            )));
        };

        // no args?
        if func.arg_count == 0 {
            let result = if let Ok((input, _)) = end_of_statement(input) {
                // no args
                Ok((
                    input,
                    Self {
                        name: identifier.to_string(),
                        args: Vec::new(),
                    },
                ))
            } else {
                Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)))
            };

            return result;
        }

        // has args
        let (input, _) = ((not(end_of_statement), ws)).parse(input)?;

        let (mut input, mut args) = {
            let (input, (first_arg, _)) = ((Expression::parse, ws)).parse(input)?;
            (input, vec![first_arg])
        };

        // grab arguments
        for _ in 0..func.arg_count - 1 {
            // TODO for expression, implement some way to either make the expression parse until the end of the statement or stringify the expression
            let (input_new, (_, _, expr, _)) =
                tuple((character::complete::char(','), ws, Expression::parse, ws))(input)?;
            input = input_new;
            args.push(expr);
        }

        Ok((
            input,
            Self {
                name: identifier.to_string(),
                args,
            },
        ))
    }
}

#[derive(Debug, Clone)]
/// A function definition
pub struct FunctionDef {
    pub name: String,
    pub arg_count: usize,
    pub body: usize,
    pub body_line: usize,
}

const FUNCTION_HEADER: &[char] = &['f', 'u', 'n', 'c', 't', 'i', 'o', 'n'];

impl FunctionDef {
    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> AstParseResult<'a, Self> {
        // header
        let (input, first_ch) = satisfy(|c| !is_ws(c))(input)?;
        let header_start_index = FUNCTION_HEADER.iter().position(|c| *c == first_ch);
        let Some(header_start_index) = header_start_index else {
            return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
        };

        let (input, rest) = chunk(input)?;
        if FUNCTION_HEADER.len() < rest.input.len() - 1 {
            return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
        }

        let mut function_header = FUNCTION_HEADER.iter().skip(header_start_index + 1);
        for ch in rest.input.chars() {
            loop {
                let function_ch = function_header.next();
                let Some(function_ch) = function_ch else {
                    return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
                };
                if *function_ch == ch {
                    break;
                }
            }
        }

        // past header
        // func_args = { identifier ~ (comma ~ identifier)* }
        // ws_silent+ ~ identifier ~ (ws_silent+ ~ func_args? | ws_silent+) ~ arrow ~ ws_silent* ~ (scope_block | (expression ~ term))
        let comma = || character::complete::char(',');
        let arg_identifier = || identifier(comma());
        let args = tuple((
            arg_identifier(),
            many0_count(tuple((ws, comma(), ws, arg_identifier()))),
        ));
        let args = tuple((ws, args, ws)).map(|(_, (_, count), _)| count + 1);
        let identifier = identifier(fail::<_, Position, nom::error::Error<_>>);
        let arrow = tag("=>");

        let (input, (_, identifier, args, _, _)) =
            ((ws, identifier, opt(args), arrow, ws)).parse(input)?;

        let body = input.index;
        let body_line = input.line;

        let instance = Self {
            name: identifier.input.to_string(),
            arg_count: args.unwrap_or_default(),
            body,
            body_line,
        };

        Ok((input, instance))
    }

    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        interpreter.state.add_func(&self.name, self.into());
        Ok(())
    }
}

impl From<&FunctionDef> for Function {
    fn from(func: &FunctionDef) -> Self {
        Self {
            arg_count: func.arg_count,
            variant: FunctionVariant::FunctionDefined {
                defined_line: func.body_line,
                body_location: func.body,
            },
        }
    }
}
