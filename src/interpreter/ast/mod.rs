use pest::iterators::Pairs;

use self::{function::FunctionCall, variable::VariableDecl};

use super::{
    parser::Rule,
    runtime::{self, error::Error, value::Value},
    InterpreterState,
};

mod expression;
mod function;
mod uncertain;
mod variable;

#[derive(Debug)]
/// An abstract syntax tree that represents a scope of code
pub struct Ast<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Ast<'a> {
    /// Parse a pest parse tree into an AST
    pub fn parse(mut pairs: Pairs<'a, Rule>) -> Self {
        // program rule
        let pairs = pairs.next().unwrap().into_inner();

        let mut statements = Vec::new();

        for statement in pairs {
            // end of input check
            if statement.as_rule() == Rule::EOI {
                break;
            }

            // now should be in a statement
            let mut statement = statement.into_inner().next().unwrap().into_inner();
            let (statement, _) = (statement.next().unwrap(), statement.next().unwrap());

            // process it
            let parsed = match statement.as_rule() {
                Rule::var_var => Statement::VariableDecl(statement.into()),
                Rule::func_call => Statement::FunctionCall(statement.into()),
                _ => unreachable!(),
            };

            statements.push(parsed);
        }

        Self { statements }
    }

    pub fn eval(&self, state: &InterpreterState<'a>) -> Result<Value, Error> {
        state.push_scope();

        let mut ret_value = None;
        for statement in &self.statements {
            ret_value = statement.eval(state)?;
            if ret_value.is_some() {
                break;
            }
        }

        state.pop_scope();

        // function calls return undefined if they don't return anything
        Ok(ret_value.unwrap_or(Value::Undefined))
    }
}

#[derive(Debug)]
/// Single statement that does something
pub enum Statement<'a> {
    FunctionCall(FunctionCall<'a>),
    VariableDecl(VariableDecl<'a>),
}

impl<'a> Statement<'a> {
    /// Evaluates the statement
    pub fn eval(
        &self,
        interpreter: &InterpreterState<'a>,
    ) -> Result<Option<Value>, runtime::error::Error> {
        match self {
            Statement::FunctionCall(function) => function.eval(interpreter).map(|_| None),
            Statement::VariableDecl(decl) => decl.eval(interpreter).map(|_| None),
        }
    }
}
