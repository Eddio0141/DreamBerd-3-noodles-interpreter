//! Contains function related structures

use nom::IResult;
use pest::iterators::Pair;

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::interpreter::static_analysis::FunctionInfo;
use crate::Interpreter;

use super::expression::Expression;
use super::{Ast, ParserInput, Statement};

#[derive(Debug, Clone)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall {
    name: String,
    args: Vec<Expression>,
}

impl From<Pair<'_, super::Rule>> for FunctionCall {
    fn from(value: Pair<'_, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str().to_string();

        let args = if let Some(value) = value.peek() {
            let mut args = Vec::new();

            for arg in value.into_inner() {
                if arg.as_rule() != Rule::expression {
                    continue;
                }

                args.push(Expression::from(arg));
            }

            args
        } else {
            Vec::new()
        };

        Self { name, args }
    }
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

    pub fn parse(mut input: ParserInput) -> IResult<ParserInput, Self> {
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
    pub fn parse<'a>(code: ParserInput<'a>) -> IResult<ParserInput<'a>, Self> {
        todo!()
    }
}

impl From<Pair<'_, Rule>> for FunctionDef {
    fn from(value: Pair<'_, Rule>) -> Self {
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str().to_string();
        let next = value.next().unwrap();

        let (args, next) = if next.as_rule() == Rule::func_args {
            let mut args = Vec::new();
            let inner_value = next.into_inner();

            for arg in inner_value {
                args.push(arg.as_str().to_string());
            }

            (args, value.next().unwrap())
        } else {
            (Vec::new(), next)
        };

        let body = match next.as_rule() {
            Rule::scope_block => FunctionVariant::Ast(next.into_inner().into()),
            Rule::expression => FunctionVariant::Expression(next.into()),
            _ => unreachable!("Unexpected rule: {:?}", next.as_rule()),
        };

        Self { name, args, body }
    }
}

#[derive(Debug, Clone)]
pub enum FunctionVariant {
    Ast(Ast),
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
            FunctionVariant::Ast(ast) => ast.eval_func(interpreter, arg_names, args),
            FunctionVariant::Expression(expr) => expr.eval(interpreter),
        }
    }
}
