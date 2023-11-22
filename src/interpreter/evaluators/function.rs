//! Contains function related structures

use nom::character::{self, complete};
use nom::combinator::fail;
use nom::multi::separated_list0;
use nom::sequence::Tuple;

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::parsers::identifier;
use crate::parsers::types::Position;
use crate::Interpreter;

use super::expression::Expression;
use super::parsers::EvalResult;

#[derive(Debug, Clone)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall {
    name: String,
    args: Vec<Expression>,
}

impl FunctionCall {
    pub fn eval<'a>(&'a self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(arg.eval(interpreter)?);
        }
        let args = args.iter().collect::<Vec<_>>();

        interpreter.state.invoke_func(interpreter, &self.name, args)
    }

    pub fn parse<'a>(input: Position<'a, &'a Interpreter<'a>>) -> EvalResult<'a, Self> {
        // function call syntax
        // - `func_name!`
        // with args
        // - `func_name arg1, arg2!`
        let mut identifier = identifier(fail::<_, Position<&Interpreter>, _>);
        // let comma = character::complete::char(',');
        let (input, identifier) = identifier(input)?;

        // does the function exist
        if let Some(func) = input.extra.state.get_func_info(identifier.into()) {

        }

        todo!()
    }
}

#[derive(Debug, Clone)]
/// A function definition
pub struct FunctionDef<'a> {
    pub name: &'a str,
    pub arg_count: usize,
    pub body: FunctionVariant,
}

impl<'a> FunctionDef<'a> {
    pub fn parse(code: &'a str) -> EvalResult<'a, Self> {
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
