//! Contains function related structures

use std::cell::Ref;

use nom::{
    branch::alt, bytes::complete::*, character::complete::*, combinator::*, error::ErrorKind,
    multi::*, sequence::*, *,
};

use crate::{
    interpreter::{
        evaluators::statement::Statement,
        runtime::{
            error::Error,
            state::{DefineType, Function, FunctionVariant},
            value::Value,
        },
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
    pub fn eval(&self, interpreter: &Interpreter, code: &str) -> Result<Value, Error> {
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(arg.eval(interpreter, code)?);
        }

        interpreter
            .state
            .invoke_func(interpreter, code, &self.name, args)
    }

    fn try_get_func<'a, 'b, 'c, P, PO>(
        input: Position<'a, 'b, Interpreter<'c>>,
        identifier_term: P,
        fail_if_lower_identifier_order: bool,
    ) -> IResult<Position<'a, 'b, Interpreter<'c>>, (&'a str, Ref<'b, Function>), ()>
    where
        P: Parser<Position<'a, 'b, Interpreter<'c>>, PO, ()>,
    {
        let mut identifier = identifier(identifier_term);

        let (input, identifier) = identifier(input)?;
        let identifier = identifier.into();

        let func = if fail_if_lower_identifier_order {
            if let Some(DefineType::Func(func)) = input.extra.state.get_identifier(identifier) {
                func
            } else {
                return Err(nom::Err::Error(()));
            }
        } else {
            if let Some(func) = input.extra.state.get_func_info(identifier) {
                func
            } else {
                return Err(nom::Err::Error(()));
            }
        };

        // does the function exist
        Ok((input, (identifier, func)))
    }

    pub fn parse_maybe_as_func<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        Self::parse(input, true)
    }

    pub fn parse_as_func<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        Self::parse(input, false)
    }

    fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
        fail_if_lower_identifier_order: bool,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        // function call syntax
        // - `func_name!`
        // with args
        // - `func_name arg1, arg2!`

        // try a stricter one first, and the relaxed after
        let (input, (identifier, func)) =
            if let Ok(res) = Self::try_get_func(input, char('!'), fail_if_lower_identifier_order) {
                res
            } else {
                Self::try_get_func(input, fail::<_, (), _>, fail_if_lower_identifier_order)
                    .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Fail)))?
            };

        // no args?
        if func.arg_count == 0 {
            // no args
            return Ok((
                input,
                Self {
                    name: identifier.to_string(),
                    args: Vec::new(),
                },
            ));
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
                tuple((char(','), ws, Expression::parse, ws))(input)?;
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
    pub args: Vec<String>,
    pub body: String,
    pub body_line: usize,
}

const FUNCTION_HEADER: &[char] = &['f', 'u', 'n', 'c', 't', 'i', 'o', 'n'];

impl FunctionDef {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        // header
        let (input, first_ch) = satisfy(|c| !is_ws(c))(input)?;
        let header_start_index = FUNCTION_HEADER.iter().position(|c| *c == first_ch);
        let Some(header_start_index) = header_start_index else {
            return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
        };

        let (input, rest) = chunk(input)?;

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
        let comma = || char::<Position<_, _>, _>(',');
        let arg_identifier = || identifier(comma());
        let args = tuple((
            arg_identifier(),
            many0(
                tuple((ws, comma(), ws, arg_identifier())).map(|(_, _, _, identifier)| identifier),
            ),
        ))
        .map(|(first, mut rest)| {
            rest.insert(0, first);
            rest
        });
        let arrow = || tag("=>");
        let args = tuple((ws, args, ws, arrow())).map(|(_, args, _, _)| {
            args.into_iter()
                .map(|s| s.input.to_string())
                .collect::<Vec<_>>()
        });
        let identifier = identifier(arrow());
        let scope_start = char('{');
        let scope_end = char('}');
        let scope = tuple((scope_start, ws, many0(Statement::parse), scope_end));
        let scope = scope.map(|_| ());
        let expression = tuple((Expression::parse, end_of_statement)).map(|_| ());

        let (body, (_, identifier, _, args, _)) = ((
            ws,
            identifier,
            ws,
            alt((arrow().map(|_| Vec::new()), args)),
            ws,
        ))
            .parse(input)?;

        let body_line = body.line;

        let (input, _) = alt((expression, scope))(body)?;

        let instance = Self {
            name: identifier.input.to_string(),
            args,
            body: body.input.to_string(),
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
            arg_count: func.args.len(),
            variant: FunctionVariant::FunctionDefined {
                defined_line: func.body_line,
                body: func.body.clone(),
                args: func.args.clone(),
            },
        }
    }
}
