use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    interpreter::{
        evaluators::{
            expression::{AtomPostfix, Expression},
            statement::Statement,
            EvalArgs,
        },
        static_analysis::{Analysis, FunctionInfo},
    },
    parsers::types::Position,
    prelude::Wrapper,
    runtime::{stdlib::array, value::PROTO_PROP},
    Interpreter,
};

use super::{
    error::Error,
    stdlib::function,
    value::{Object, Value},
};

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
    pub vars: Arc<Mutex<CallStack<Scope<VariableState>>>>,
    // functions are either global or declared as a variable
    pub funcs: Arc<Mutex<CallStack<Scope<Functions>>>>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        let var_state = VariableState::default();
        let func_state = Functions::default();

        let vars = vec![var_state];
        let funcs = vec![func_state];

        let vars = vec![vars];
        let funcs = vec![funcs];

        Self {
            vars: Arc::new(Mutex::new(vars)),
            funcs: Arc::new(Mutex::new(funcs)),
        }
    }
}

impl InterpreterState {
    /// Gets function info
    pub fn get_func_info(&self, name: &str) -> Option<FunctionState> {
        self.funcs
            .lock()
            .unwrap()
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
            let func = self.add_func(
                identifier,
                FunctionVariant::FunctionDefined {
                    defined_line: hoisted_line,
                    body: Arc::new(code[body_location..].to_string()),
                    arg_names: Arc::new(args.iter().map(|s| s.to_string()).collect()),
                },
                Some(args.len()),
            );
            self.add_var(identifier, func.into(), hoisted_line);
        }
    }

    pub fn push_scope(&self, line: Option<usize>) {
        let (mut vars, mut funcs) = (self.vars.lock().unwrap(), self.funcs.lock().unwrap());
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

        funcs.push(Functions(new_scope));
    }

    pub fn pop_scope(&self, line: Option<usize>) {
        let (mut vars, mut funcs) = (self.vars.lock().unwrap(), self.funcs.lock().unwrap());
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
            .lock()
            .unwrap()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .declare_var(name, value, line);
    }

    pub fn get_var(&self, name: &str) -> Option<Variable> {
        self.vars.lock().unwrap().iter().find_map(|vars| {
            vars.iter()
                .rev()
                .find_map(|vars| vars.get_var(name).cloned())
        })
    }

    pub fn set_var(
        &self,
        name: &str,
        args: EvalArgs,
        postfix: &[AtomPostfix],
        value: Value,
        line: usize,
    ) -> Result<(), Error> {
        let mut vars = self.vars.lock().unwrap();
        for vars in vars.iter_mut() {
            let vars_iter = vars.iter_mut().rev();

            for vars in vars_iter {
                if vars.set_var(name, args, postfix, &value)? {
                    return Ok(());
                }
            }
        }

        // declare global
        vars.first_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .declare_var(name, value, line);

        Ok(())
    }

    /// Part of the function constructor
    /// - This declares a function and binds it to an object
    /// # Arg count
    /// - If you pass `None`, it can accept any number of arguments
    pub fn add_func(
        &self,
        name: &str,
        func: FunctionVariant,
        arg_count: Option<usize>,
    ) -> Arc<Mutex<Object>> {
        let mut properties = HashMap::new();
        properties.insert(
            PROTO_PROP.to_string(),
            Arc::clone(&function::PROTOTYPE).into(),
        );
        let obj = Object::new(properties);
        let obj = Arc::new(Mutex::new(obj));

        let state = FunctionState {
            arg_count,
            variant: func,
            obj: Arc::clone(&obj),
        };
        self.funcs
            .lock()
            .unwrap()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .0
            .insert(name.to_string(), state);

        obj
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
    Func(FunctionState),
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

    pub fn set_var(
        &mut self,
        name: &str,
        args: EvalArgs,
        postfix: &[AtomPostfix],
        value: &Value,
    ) -> Result<bool, Error> {
        if let Some(var) = self.0.get_mut(name) {
            var.set_value(args, value.clone(), postfix)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    value: Value,
    line: usize,
}

impl Variable {
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn set_value(
        &mut self,
        args: EvalArgs,
        value: Value,
        postfix: &[AtomPostfix],
    ) -> Result<(), Error> {
        if postfix.is_empty() {
            self.value = value;
            return Ok(());
        }

        let mut var = Cow::Borrowed(&self.value);
        let postfix_last = postfix.last().unwrap();
        for i in 0..postfix.len() - 1 {
            let postfix = &postfix[i];
            var = postfix.eval(var, args)?.0;
        }

        let Value::Object(var) = var.into_owned() else {
            return Ok(());
        };

        // TODO concrete error
        let Some(var) = var else {
            return Err(Error::Type("Cannot read propertoes of null".to_string()));
        };

        let mut var = var.lock().unwrap();

        // TODO reuse code from postfix
        match postfix_last {
            AtomPostfix::DotNotation(identifier) => var.set_property(identifier, value),
            AtomPostfix::BracketNotation(expr) => {
                if let Value::String(key) = expr.eval(args)?.0.borrow() {
                    var.set_property(key, value)
                } else {
                    todo!()
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
/// A stack of function states
pub struct Functions(pub HashMap<String, FunctionState>);

#[derive(Debug, Clone)]
/// Function state
/// - A function is either
///   - binded to an object, which is the object that the function is called on
///   - it is a native function
pub struct FunctionState {
    pub arg_count: Option<usize>,
    variant: FunctionVariant,
    obj: Arc<Mutex<Object>>,
}

impl FunctionState {
    fn eval(&self, eval_args: EvalArgs, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
        let interpreter = eval_args.1.extra;
        let state = &interpreter.state;

        state.funcs.lock().unwrap().push(vec![Functions::default()]);
        state
            .vars
            .lock()
            .unwrap()
            .push(vec![VariableState::default()]);

        match &self.variant {
            FunctionVariant::FunctionDefined {
                body,
                arg_names,
                defined_line: _,
            } => {
                let mut obj = self.obj.lock().unwrap();

                obj.set_property("arguments", array::constructor(interpreter, args)?);
                let Value::Object(Some(args)) = obj.get_property("arguments").unwrap() else {
                    unreachable!();
                };
                let args = args.lock().unwrap();

                let pop_call_stack = || {
                    state.funcs.lock().unwrap().pop();
                    state.vars.lock().unwrap().pop();
                };

                state.funcs.lock().unwrap().push(vec![Functions::default()]);
                state
                    .vars
                    .lock()
                    .unwrap()
                    .push(vec![VariableState::default()]);

                // declare arguments
                for (arg_name, arg_value) in arg_names.iter().zip(args.array_obj_iter()) {
                    state.add_var(&arg_name.to_string(), arg_value, 0);
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
                        let ret = match statement.eval(eval_args) {
                            Ok(ret) => ret.return_value,
                            Err(err) => return Err(err),
                        };
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
        body: Arc<String>,
        /// Argument names
        arg_names: Arc<Vec<String>>,
    },
    Native(NativeFunc),
}

pub type NativeFunc = fn(&Interpreter<'_>, Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error>;
