use std::{cell::RefCell, collections::HashMap};

use crate::{interpreter::ast::Ast, Interpreter};

use super::{error::Error, value::Value};

#[derive(Debug)]
/// Interpreter state
pub struct InterpreterState {
    vars: RefCell<Vec<VariableState>>,
    funcs: RefCell<Vec<FunctionState>>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        Self {
            vars: RefCell::new(vec![Default::default()]),
            funcs: RefCell::new(vec![Default::default()]),
        }
    }
}

impl InterpreterState {
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
        interpreter: &Interpreter<'_>,
        name: &str,
        args: Vec<&Value>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.funcs.borrow().iter().find_map(|func| func.0.get(name)) {
            return func.eval(interpreter, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn add_var(&self, name: &str, value: Value) {
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
            .find_map(|vars| vars.get_var(name).cloned())
    }

    pub fn set_var(&self, name: &str, value: Value) {
        let mut vars = self.vars.borrow_mut();
        let vars_iter = vars.iter_mut().rev();

        for vars in vars_iter {
            if vars.set_var(name, &value).is_some() {
                return;
            }
        }

        // declare global
        vars.last_mut().unwrap().declare_var(name, value);
    }

    pub fn add_func(&self, name: &str, func: FunctionVariant) {
        self.funcs
            .borrow_mut()
            .last_mut()
            .unwrap()
            .0
            .insert(name.to_string(), func);
    }
}

#[derive(Debug, Default)]
pub struct VariableState(pub HashMap<String, Value>);

impl VariableState {
    pub fn declare_var(&mut self, name: &str, value: Value) {
        self.0.insert(name.to_owned(), value);
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    pub fn set_var(&mut self, name: &str, value: &Value) -> Option<()> {
        if let Some(var) = self.0.get_mut(name) {
            *var = value.clone();
            Some(())
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct FunctionState(pub HashMap<String, FunctionVariant>);

impl FunctionState {
    pub fn invoke_func(
        &self,
        name: &str,
        interpreter: &Interpreter,
        args: Vec<&Value>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.0.get(name) {
            return func.eval(interpreter, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }
}

#[derive(Debug)]
pub enum FunctionVariant {
    Ast(Ast),
    Native(fn(&Interpreter, Vec<&Value>) -> Result<Value, Error>),
}

impl FunctionVariant {
    pub fn eval(&self, interpreter: &Interpreter<'_>, args: Vec<&Value>) -> Result<Value, Error> {
        match self {
            FunctionVariant::Ast(ast) => ast.eval(interpreter, false),
            FunctionVariant::Native(func) => func(interpreter, args),
        }
    }
}
