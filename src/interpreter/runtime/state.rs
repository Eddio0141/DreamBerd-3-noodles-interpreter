use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        evaluators::{expression::Expression, statement::Statement, EvalArgs},
        static_analysis::{Analysis, FunctionInfo},
    },
    parsers::types::Position,
    prelude::Wrapper,
    Interpreter,
};

use super::{error::Error, value::Value};

type Scope<T> = Vec<T>;
type CallStack<T> = Vec<T>;

#[derive(Debug)]
/// Interpreter state
pub struct InterpreterState {
    /*
     * by having a vec for each function call (CallStack), function call is as easy as pushing and popping
     * without keeping track of how many scopes were opened which the `return` statement
     * will have to pop on early return
     */
    vars: Rc<RefCell<CallStack<Scope<VariableState>>>>,
    // functions are either global or declared as a variable
    funcs: Rc<RefCell<CallStack<Scope<FunctionState>>>>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        let var_state = VariableState::default();
        let func_state = FunctionState::default();

        let vars = vec![var_state];
        let funcs = vec![func_state];

        let vars = vec![vars];
        let funcs = vec![funcs];

        Self {
            vars: Rc::new(RefCell::new(vars)),
            funcs: Rc::new(RefCell::new(funcs)),
        }
    }
}

impl InterpreterState {
    /// Gets function info
    pub fn get_func_info(&self, name: &str) -> Option<Function> {
        let funcs = self.funcs.borrow();
        funcs
            .iter()
            .find_map(|funcs| funcs.iter().find_map(|funcs| funcs.0.get(name)))
            .cloned()
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
                        body: Rc::new(code[body_location..].to_string()),
                        args: Rc::new(args.iter().map(|s| s.to_string()).collect()),
                    },
                },
            )
        }
    }

    pub fn push_scope(&self, line: Option<usize>) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());
        let (vars, funcs) = (vars.last_mut().unwrap(), funcs.last_mut().unwrap());

        vars.push(Default::default());

        // when pushing scope, the hoisted functions that's defined after the push position will be pushed up to the new scope
        let last_scope = &mut funcs.last_mut().unwrap().0;
        let mut new_scope = HashMap::new();

        match line {
            Some(line) => last_scope.retain(|name, func| {
                if let FunctionVariant::FunctionDefined { defined_line, .. } = &func.variant {
                    if *defined_line > line {
                        new_scope.insert(name.to_string(), func.clone());
                    }
                    return false;
                }

                true
            }),
            None => new_scope.extend(last_scope.drain()),
        }

        funcs.push(FunctionState(new_scope));
    }

    pub fn pop_scope(&self, line: Option<usize>) {
        let (mut vars, mut funcs) = (self.vars.borrow_mut(), self.funcs.borrow_mut());
        let (vars, funcs) = (vars.last_mut().unwrap(), funcs.last_mut().unwrap());

        if vars.len() == 1 {
            return;
        }

        vars.pop();

        // opposite to push_scope with hoisted functions
        let remove_scope = funcs.pop().unwrap().0;
        let last_scope = &mut funcs.last_mut().unwrap().0;
        match line {
            Some(line) => {
                for (name, func) in remove_scope {
                    if let FunctionVariant::FunctionDefined { defined_line, .. } = &func.variant {
                        if *defined_line > line {
                            last_scope.insert(name, func);
                        }
                    }
                }
            }
            None => last_scope.extend(remove_scope),
        }
    }

    pub fn invoke_func(
        &self,
        eval_args: EvalArgs,
        name: &str,
        args: Vec<Wrapper<Cow<Value>>>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.get_func_info(name) {
            return func.eval(eval_args, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn add_var(&self, name: &str, value: Value, line: usize) {
        self.vars
            .borrow_mut()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .declare_var(name, value, line);
    }

    pub fn get_var(&self, name: &str) -> Option<Variable> {
        self.vars.borrow().iter().find_map(|vars| {
            vars.iter()
                .rev()
                .find_map(|vars| vars.get_var(name).cloned())
        })
    }

    pub fn set_var(&self, name: &str, value: Value, line: usize) {
        let mut vars = self.vars.borrow_mut();
        for vars in vars.iter_mut() {
            let vars_iter = vars.iter_mut().rev();

            for vars in vars_iter {
                if vars.set_var(name, &value).is_some() {
                    return;
                }
            }
        }

        // declare global
        vars.first_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .declare_var(name, value, line);
    }

    pub fn add_func(&self, name: &str, func: Function) {
        self.funcs
            .borrow_mut()
            .last_mut()
            .unwrap()
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

pub enum DefineType {
    Var(Variable),
    Func(Function),
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
    fn eval(&self, eval_args: EvalArgs, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
        let interpreter = eval_args.1.extra;
        let state = &interpreter.state;

        let pop_call_stack = || {
            state.funcs.borrow_mut().pop();
            state.vars.borrow_mut().pop();
        };

        match &self.variant {
            FunctionVariant::FunctionDefined {
                body,
                args: arg_names,
                defined_line: _,
            } => {
                state
                    .funcs
                    .borrow_mut()
                    .push(vec![FunctionState::default()]);
                state.vars.borrow_mut().push(vec![VariableState::default()]);

                // declare arguments
                for (arg_name, arg_value) in arg_names.iter().zip(args) {
                    state.add_var(arg_name, arg_value.0.into_owned(), 0);
                }

                let code_with_pos = Position::new_with_extra(body.as_str(), interpreter);
                let eval_args = (eval_args.0, code_with_pos);

                // check if block
                if let Ok((mut code_with_pos, Statement::ScopeStart(_))) =
                    Statement::parse(code_with_pos)
                {
                    let mut scope_count = 1usize;

                    // its a block
                    while let Ok((code_after, statement)) = Statement::parse(code_with_pos) {
                        match statement {
                            Statement::ScopeStart(_) => {
                                scope_count =
                                    scope_count.checked_add(1).expect("scope count overflow")
                            }
                            Statement::ScopeEnd(_) => {
                                scope_count -= 1;
                                if scope_count == 0 {
                                    break;
                                }
                            }
                            _ => (),
                        }

                        code_with_pos = code_after;
                        let ret = statement.eval(eval_args)?.return_value;
                        if let Some(ret) = ret {
                            pop_call_stack();
                            return Ok(ret);
                        }
                    }

                    pop_call_stack();
                    return Ok(Value::Undefined);
                }

                // expression (this won't fail because implicit strings)
                if let Ok((_, expression)) = Expression::parse(code_with_pos) {
                    let value = expression.eval(eval_args)?;
                    pop_call_stack();
                    return Ok(value.0.into_owned());
                }

                unreachable!("function body is not a block or expression, which should be impossible because of implicit strings");
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
        body: Rc<String>,
        /// The arguments of the function
        args: Rc<Vec<String>>,
    },
    Native(NativeFunc),
}

pub type NativeFunc = fn(&Interpreter<'_>, Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error>;
