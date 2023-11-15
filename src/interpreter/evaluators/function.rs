//! Contains function related structures

use nom::IResult;

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::Interpreter;

use super::expression::Expression;

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
        let args = args.iter().collect::<Vec<_>>();

        interpreter.state.invoke_func(interpreter, &self.name, args)
    }

    pub fn parse<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
        todo!()
    }
}

#[derive(Debug, Clone)]
/// A function definition
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<String>,
    pub body: FunctionVariant,
}

impl FunctionDef {
    pub fn parse<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
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
        match self {
            // FunctionVariant::Ast(ast) => ast.eval_func(interpreter, arg_names, args),
            FunctionVariant::Expression(expr) => expr.eval(interpreter),
        }
    }
}
