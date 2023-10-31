use std::{cell::RefCell, collections::HashMap};

use crate::{interpreter::ast::Ast, Interpreter};

use super::{error::Error, value::Value};

#[derive(Debug)]
/// Interpreter state
pub struct InterpreterState<'a> {
    vars: RefCell<Vec<VariableState<'a>>>,
    funcs: RefCell<Vec<FunctionState<'a>>>,
}

impl Default for InterpreterState<'_> {
    fn default() -> Self {
        Self {
            vars: RefCell::new(vec![Default::default()]),
            funcs: RefCell::new(vec![Default::default()]),
        }
    }
}

impl<'a> InterpreterState<'a> {
    pub fn push_scope(&self) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());

        vars.push(Default::default());
        funcs.push(Default::default());
    }

    pub fn pop_scope(&self) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());

        vars.pop();
        funcs.pop();
    }

    pub fn invoke_func(
        &self,
        interpreter: &Interpreter<'a>,
        name: &'a str,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.funcs.borrow().iter().find_map(|func| func.0.get(name)) {
            return func.eval(interpreter, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn declare_var(&self, name: &'a str, value: Value) {
        self.vars
            .borrow_mut()
            .last_mut()
            .unwrap()
            .declare_var(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        self.vars
            .borrow()
            .iter()
            .rev()
            .find_map(|vars| vars.get_var(name).map(|value| *value))
    }

    pub fn add_func(&self, name: &'a str, func: FunctionVariant<'a>) {
        self.funcs
            .borrow_mut()
            .last_mut()
            .unwrap()
            .0
            .insert(name, func);
    }
}

#[derive(Debug, Default)]
pub struct VariableState<'a>(pub HashMap<&'a str, Value>);

impl<'a> VariableState<'a> {
    pub fn declare_var(&mut self, name: &'a str, value: Value) {
        self.0.insert(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    pub fn set_var(&mut self, name: &'a str, value: Value) {
        self.0.insert(name, value);
    }
}

#[derive(Debug, Default)]
pub struct FunctionState<'a>(pub HashMap<&'a str, FunctionVariant<'a>>);

impl<'a> FunctionState<'a> {
    pub fn invoke_func(
        &self,
        name: &'a str,
        interpreter: &Interpreter<'a>,
        args: Vec<Value>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.0.get(name) {
            return func.eval(interpreter, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }
}

#[derive(Debug)]
pub enum FunctionVariant<'a> {
    Ast(Ast<'a>),
    Native(fn(&Interpreter<'a>, Vec<Value>) -> Result<Value, Error>),
}

impl<'a> FunctionVariant<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>, args: Vec<Value>) -> Result<Value, Error> {
        match self {
            FunctionVariant::Ast(ast) => ast.eval(interpreter),
            FunctionVariant::Native(func) => func(interpreter, args),
        }
    }
}
