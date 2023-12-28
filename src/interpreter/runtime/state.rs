use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    collections::HashMap,
};

use crate::{
    interpreter::{
        evaluators::{expression::Expression, statement::Statement},
        static_analysis::{Analysis, FunctionInfo},
    },
    parsers::types::Position,
    prelude::Wrapper,
    Interpreter,
};

use super::{error::Error, value::Value};

#[derive(Debug)]
/// Interpreter state
pub struct InterpreterState {
    vars: RefCell<Vec<VariableState>>,
    // functions are either global or declared as a variable
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
    /// Gets function info
    pub fn get_func_info(&self, name: &str) -> Option<Ref<Function>> {
        let funcs = self.funcs.borrow();
        Ref::filter_map(funcs, |funcs| {
            funcs.iter().find_map(|funcs| funcs.0.get(name))
        })
        .ok()
    }

    /// Adds the analysis information to the state
    pub fn add_analysis_info(&self, code: &str, analysis: Analysis) {
        for func in analysis.hoisted_funcs {
            let FunctionInfo {
                identifier,
                args,
                hoisted_line,
                body_location,
            } = func;
            self.add_func(
                identifier,
                Function {
                    arg_count: args.len(),
                    variant: FunctionVariant::FunctionDefined {
                        defined_line: hoisted_line,
                        body: code[body_location..].to_string(),
                        args: args.iter().map(|s| s.to_string()).collect(),
                    },
                },
            )
        }
    }

    pub fn push_scope(&self, line: usize) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());

        vars.push(Default::default());

        // when pushing scope, the hoisted functions that's defined after the push position will be pushed up to the new scope
        let last_scope = &mut funcs.last_mut().unwrap().0;
        let mut new_scope = HashMap::new();
        last_scope.retain(|name, func| {
            if let FunctionVariant::FunctionDefined { defined_line, .. } = &func.variant {
                if *defined_line > line {
                    new_scope.insert(name.to_string(), func.clone());
                }
                return false;
            }

            true
        });
        funcs.push(FunctionState(new_scope));
    }

    pub fn pop_scope(&self, line: usize) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());

        if vars.len() == 1 {
            return;
        }

        vars.pop();

        // opposite to push_scope with hoisted functions
        let remove_scope = funcs.pop().unwrap();
        let last_scope = &mut funcs.last_mut().unwrap().0;
        for (name, func) in remove_scope.0 {
            if let FunctionVariant::FunctionDefined { defined_line, .. } = &func.variant {
                if *defined_line > line {
                    last_scope.insert(name, func);
                }
            }
        }
    }

    pub fn invoke_func(
        &self,
        interpreter: &Interpreter<'_>,
        code: &str,
        name: &str,
        args: Vec<Wrapper<Cow<Value>>>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.get_func_info(name) {
            return func.eval(interpreter, code, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn add_var(&self, name: &str, value: Value, line: usize) {
        self.vars
            .borrow_mut()
            .last_mut()
            .unwrap()
            .declare_var(name, value, line);
    }

    fn get_var(&self, name: &str) -> Option<Variable> {
        self.vars
            .borrow()
            .iter()
            .rev()
            .find_map(|vars| vars.get_var(name).cloned())
    }

    pub fn set_var(&self, name: &str, value: Value, line: usize) {
        let mut vars = self.vars.borrow_mut();
        let vars_iter = vars.iter_mut().rev();

        for vars in vars_iter {
            if vars.set_var(name, &value).is_some() {
                return;
            }
        }

        // declare global
        vars.last_mut().unwrap().declare_var(name, value, line);
    }

    pub fn add_func(&self, name: &str, func: Function) {
        self.funcs
            .borrow_mut()
            .last_mut()
            .unwrap()
            .0
            .insert(name.to_string(), func);
    }

    /// Tries to get the latest defined variable or function with the given name
    pub fn get_identifier(&self, name: &str) -> Option<DefineType> {
        // check functions first
        let func = self.get_func_info(name);
        let var = self.get_var(name);

        let Some(func) = func else {
            return var.map(DefineType::Var);
        };

        let Some(var) = var else {
            return Some(DefineType::Func(func));
        };

        let ret = match func.variant {
            FunctionVariant::FunctionDefined { defined_line, .. } => {
                if var.line > defined_line {
                    DefineType::Var(var)
                } else {
                    DefineType::Func(func)
                }
            }
            FunctionVariant::Native(_) => DefineType::Var(var),
        };

        Some(ret)
    }
}

pub enum DefineType<'a> {
    Var(Variable),
    Func(Ref<'a, Function>),
}

#[derive(Debug, Default)]
pub struct VariableState(pub HashMap<String, Variable>);

impl VariableState {
    pub fn declare_var(&mut self, name: &str, value: Value, line: usize) {
        self.0.insert(name.to_string(), Variable { value, line });
    }

    pub fn get_var(&self, name: &str) -> Option<&Variable> {
        self.0.get(name)
    }

    pub fn set_var(&mut self, name: &str, value: &Value) -> Option<()> {
        if let Some(var) = self.0.get_mut(name) {
            var.value = value.clone();
            Some(())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub value: Value,
    line: usize,
}

#[derive(Debug, Default)]
pub struct FunctionState(pub HashMap<String, Function>);

#[derive(Debug, Clone)]
pub struct Function {
    pub arg_count: usize,
    pub variant: FunctionVariant,
}

impl Function {
    fn eval(
        &self,
        interpreter: &Interpreter,
        code: &str,
        args: Vec<Wrapper<Cow<Value>>>,
    ) -> Result<Value, Error> {
        match &self.variant {
            FunctionVariant::FunctionDefined {
                body,
                args: arg_names,
                defined_line: _,
            } => {
                // declare arguments
                for (arg_name, arg_value) in arg_names.iter().zip(args) {
                    interpreter
                        .state
                        .add_var(arg_name, arg_value.0.into_owned(), 0);
                }

                let mut code_with_pos = Position::new_with_extra(body.as_str(), interpreter);

                // try expression first (it could be a function)
                if let Ok((_, expression)) = Expression::parse(code_with_pos) {
                    let value = expression.eval(interpreter, code)?;
                    return Ok(value.0.into_owned());
                }

                // its a block
                while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
                    code_with_pos = code_after;
                    statement.eval(interpreter, code)?;
                }

                Ok(Value::Undefined)
            }
            FunctionVariant::Native(native) => native(interpreter, args),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FunctionVariant {
    FunctionDefined {
        /// The line where the function is usable from
        defined_line: usize,
        /// Where the expression / scope is located as an index
        body: String,
        /// The arguments of the function
        args: Vec<String>,
    },
    Native(NativeFunc),
}

pub type NativeFunc = fn(&Interpreter<'_>, Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error>;
