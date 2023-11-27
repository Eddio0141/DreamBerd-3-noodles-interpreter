//! Contains function related structures

use nom::combinator::{fail, not};
use nom::error::ErrorKind;
use nom::sequence::{tuple, Tuple};
use nom::{character, Err};

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::parsers::types::Position;
use crate::parsers::{end_of_statement, identifier, ws};
use crate::Interpreter;

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
                Err(Err::Failure(nom::error::Error::new(input, ErrorKind::Fail)))
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
    pub body: FunctionVariant,
}

impl FunctionDef {
    pub fn parse<'a>(code: &str) -> AstParseResult<'a, Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum FunctionVariant {
    // Ast(Ast),
    Expression(Expression),
}

impl FunctionVariant {
    pub fn eval(
        &self,
        interpreter: &Interpreter,
        arg_names: Vec<&str>,
        args: Vec<&Value>,
    ) -> Result<Value, Error> {
        // match self {
        //     // FunctionVariant::Ast(ast) => ast.eval_func(interpreter, arg_names, args),
        //     FunctionVariant::Expression(expr) => expr.eval(interpreter),
        // }
        todo!()
    }
}
