use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

use crate::{
    interpreter::static_analysis::{Analysis, FunctionInfo},
    Interpreter,
};

use super::{error::Error, value::Value};

#[derive(Debug)]
/// Interpreter state
pub struct InterpreterState<'a> {
    vars: RefCell<Vec<VariableState<'a>>>,
    // functions are either global or declared as a variable
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
    /// Gets function info
    pub fn get_func_info(&self, name: &'a str) -> Option<Ref<Function>> {
        let funcs = self.funcs.borrow();
        Ref::filter_map(funcs, |funcs| {
            funcs.iter().find_map(|funcs| funcs.0.get(name))
        })
        .ok()
    }

    /// Adds the analysis information to the state
    pub fn add_analysis_info(&self, analysis: Analysis<'a>) {
        for func in analysis.hoisted_funcs {
            let FunctionInfo {
                identifier,
                arg_count,
                hoisted_line,
                body_location,
            } = func;
            self.add_func(
                identifier,
                Function {
                    arg_count,
                    variant: FunctionVariant::FunctionDefined {
                        defined_line: hoisted_line,
                        body_location,
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
                    new_scope.insert(*name, *func);
                    return false;
                }
            }

            true
        });
        funcs.push(FunctionState(new_scope));
    }

    pub fn pop_scope(&self, line: usize) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());

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
        name: &'a str,
        args: Vec<&Value>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.get_func_info(name) {
            return func.eval(interpreter, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn add_var(&self, name: &'a str, value: Value) {
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

    pub fn set_var(&self, name: &'a str, value: Value) {
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

    pub fn add_func(&self, name: &'a str, func: Function) {
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
pub struct FunctionState<'a>(pub HashMap<&'a str, Function>);

#[derive(Debug, Clone, Copy)]
pub struct Function {
    pub arg_count: usize,
    pub variant: FunctionVariant,
}

impl Function {
    pub fn eval(&self, interpreter: &Interpreter, args: Vec<&Value>) -> Result<Value, Error> {
        // match &self.variant {
        //     FunctionVariant::FunctionDefined {body_location, defined_line} => func.eval(
        //         interpreter,
        //         self.args.iter().map(|s| s as &str).collect::<Vec<_>>(),
        //         args,
        //     ),
        //     FunctionVariant::Native(func) => func(interpreter, args),
        // }
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionVariant {
    FunctionDefined {
        /// The line where the function is usable from
        defined_line: usize,
        /// Where the expression / scope is located as an index
        body_location: usize,
    },
    Native(fn(&Interpreter, Vec<&Value>) -> Result<Value, Error>),
}
