use pest::iterators::Pairs;

use crate::Interpreter;

use self::{
    function::{FunctionCall, FunctionDef},
    variable::*,
};

use super::{
    parser::Rule,
    runtime::{self, error::Error, state::InterpreterState, value::Value},
};

mod expression;
pub mod function;
mod uncertain;
mod variable;

#[derive(Debug, Clone)]
/// An abstract syntax tree that represents a scope of code
pub struct Ast {
    funcs: Vec<FunctionDef>,
    statements: Vec<Statement>,
}

impl Ast {
    pub fn eval_global(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        self.eval_full(interpreter, true, None::<fn(&InterpreterState)>)
    }

    pub fn eval_scope(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        self.eval_full(interpreter, false, None::<fn(&InterpreterState)>)
    }

    pub fn eval_func(
        &self,
        interpreter: &Interpreter,
        arg_names: Vec<&str>,
        args: Vec<&Value>,
    ) -> Result<Value, Error> {
        let processor = |state: &InterpreterState| {
            for (name, value) in arg_names.iter().zip(args.iter()) {
                state.add_var(name, (*value).clone());
            }
        };

        self.eval_full(interpreter, false, Some(processor))
    }

    fn eval_full(
        &self,
        interpreter: &Interpreter,
        skip_scope_stack: bool,
        pre_processor: Option<impl Fn(&InterpreterState)>,
    ) -> Result<Value, Error> {
        let state = &interpreter.state;

        if !skip_scope_stack {
            state.push_scope();
        }

        for func in &self.funcs {
            state.add_func(&func.name, func.into());
        }

        if let Some(processor) = pre_processor {
            processor(state);
        }

        let mut ret_value = None;
        for statement in &self.statements {
            ret_value = statement.eval(interpreter)?;
            if ret_value.is_some() {
                break;
            }
        }

        if !skip_scope_stack {
            state.pop_scope();
        }

        // function calls return undefined if they don't return anything
        Ok(ret_value.unwrap_or(Value::Undefined))
    }
}

impl From<Pairs<'_, Rule>> for Ast {
    /// Parse a pest parse tree into an AST
    fn from(mut value: Pairs<'_, Rule>) -> Self {
        // program rule
        let pairs = value.next().unwrap().into_inner();

        let mut statements = Vec::new();
        let mut funcs = Vec::new();

        for statement in pairs {
            // end of input check
            if statement.as_rule() == Rule::EOI {
                break;
            }

            // now should be in a statement
            let statement = statement.into_inner().next().unwrap();
            let statement = match statement.as_rule() {
                Rule::statement_with_term | Rule::statement_without_term => {
                    statement.into_inner().next().unwrap()
                }
                _ => statement,
            };

            // process it
            let parsed = match statement.as_rule() {
                Rule::var_var => Statement::VariableDecl(statement.into()),
                Rule::func_call => Statement::FunctionCall(statement.into()),
                Rule::var_set => Statement::VarSet(statement.into()),
                Rule::func_def => {
                    //
                    funcs.insert(0, statement.into());
                    continue;
                }
                Rule::scope_block => Statement::ScopeBlock(statement.into_inner().into()),
                _ => unreachable!("Unexpected rule: {:?}", statement.as_rule()),
            };

            statements.push(parsed);
        }

        Self { statements, funcs }
    }
}

#[derive(Debug, Clone)]
/// Single statement that does something
pub enum Statement {
    FunctionCall(FunctionCall),
    ScopeBlock(Ast),
    VariableDecl(VariableDecl),
    VarSet(VarSet),
}

impl Statement {
    /// Evaluates the statement
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Option<Value>, runtime::error::Error> {
        match self {
            Statement::FunctionCall(function) => function.eval(interpreter).map(|_| None),
            Statement::VariableDecl(decl) => decl.eval(interpreter).map(|_| None),
            Statement::VarSet(var_set) => var_set.eval(interpreter).map(|_| None),
            Statement::ScopeBlock(ast) => ast.eval_scope(interpreter).map(|_| None),
        }
    }
}
