use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};

use crate::{
    interpreter::{
        evaluators::{
            conditional::When,
            expression::{AtomPostfix, Expression},
            statement::Statement,
            variable::VarType,
        },
        static_analysis::{Analysis, HoistedVarInfo},
    },
    parsers::{types::Position, LifeTime, PosWithInfo},
    prelude::Wrapper,
    runtime,
    runtime::{stdlib::array, value::PROTO_PROP},
    Interpreter,
};

use super::{
    error::Error,
    stdlib::function,
    value::{Object, ObjectRef, Value},
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
    pub scope_stacks: Arc<Mutex<CallStack<Scope<ScopeState>>>>,
    // function status. extra information that objects don't have
    pub funcs: Arc<Mutex<Functions>>,
    // hoisted variable info
    hoisted_vars: Arc<Mutex<Vec<HoistedVarInfo>>>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        let scope_stacks = Arc::new(Mutex::new(vec![vec![ScopeState {
            vars: VariableState::default(),
            whens: Vec::new(),
        }]]));

        Self {
            scope_stacks,
            funcs: Arc::new(Mutex::new(Functions::default())),
            hoisted_vars: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InterpreterState {
    /// Gets function info
    pub fn get_func_info(&self, name: &str, args: PosWithInfo) -> Option<FunctionState> {
        self.clean_up_funcs();
        let find_func = |value: &Value| {
            let Value::Object(Some(value)) = value else {
                return None;
            };
            // find the function
            self.funcs
                .lock()
                .unwrap()
                .0
                .iter()
                .find(|func| Weak::ptr_eq(&func.obj, &Arc::downgrade(value)))
                .cloned()
        };
        // TODO: debug when statement and find out why this is locking up
        let res = self
            .scope_stacks
            .lock()
            .unwrap()
            .iter_mut()
            .rev()
            .find_map(|scopes| {
                scopes.iter_mut().rev().find_map(|scope| {
                    let vars = &mut scope.vars;
                    vars.validate_lifetime();

                    let Some(vars) = vars.get_var(name) else {
                        return None;
                    };
                    find_func(vars.get_value())
                })
            });
        if res.is_some() {
            return res;
        }

        // check if it's hoisted
        let Ok(vars) = self.hoisted_vars.try_lock() else {
            return None;
        };
        let mut found = None;
        for var in vars.iter() {
            let value = var.eval(args)?;
            if let Some(value) = find_func(&value) {
                found = Some(value);
                break;
            }
        }

        found
    }

    /// Clean up functions that can't be called anymore
    fn clean_up_funcs(&self) {
        self.funcs
            .lock()
            .unwrap()
            .0
            .retain(|f| f.obj.upgrade().is_some());
    }

    /// Adds the analysis information to the state
    pub fn add_analysis_info(&self, analysis: Analysis) {
        *self.hoisted_vars.lock().unwrap() = analysis.hoisted_vars;
    }

    pub fn push_scope(&self, line: usize) {
        let mut scope_stacks = self.scope_stacks.lock().unwrap();
        let scopes = scope_stacks.last_mut().unwrap();

        // when pushing scope, the hoisted vars that's defined after the push position will be pushed up to the new scope
        let last_vars = &mut scopes.last_mut().unwrap().vars.0;
        let mut new_vars = HashMap::new();

        last_vars.retain(|name, var| {
            if var.line > line {
                new_vars.insert(name.to_string(), var.clone());
                false
            } else {
                true
            }
        });

        scopes.push(ScopeState {
            vars: VariableState(new_vars),
            whens: Vec::new(),
        });
    }

    pub fn pop_scope(&self, line: usize) {
        let mut scope_stacks = self.scope_stacks.lock().unwrap();
        let scopes = scope_stacks.last_mut().unwrap();
        if scopes.len() == 1 {
            return;
        }

        // opposite to push_scope with hoisted vars
        let remove_scope = scopes.pop().unwrap().vars.0;
        let last_scope = &mut scopes.last_mut().unwrap().vars.0;
        for (name, var) in remove_scope {
            if var.line > line {
                last_scope.insert(name, var);
            }
        }
    }

    pub fn invoke_func(
        &self,
        eval_args: PosWithInfo,
        name: &str,
        args: Vec<Wrapper<Cow<Value>>>,
    ) -> Result<Value, Error> {
        if let Some(func) = self.get_func_info(name, eval_args) {
            return func.eval(eval_args, args);
        }

        Err(Error::FunctionNotFound(name.to_string()))
    }

    pub fn add_var(
        &self,
        name: &str,
        value: Value,
        line: usize,
        type_: VarType,
        life_time: Option<LifeTime>,
    ) {
        self.scope_stacks
            .lock()
            .unwrap()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .vars
            .declare_var(name, value, line, type_, life_time);
    }

    // identical to add_var, but for runtime
    pub fn add_var_runtime(
        &self,
        name: &str,
        value: Value,
        line: usize,
        type_: VarType,
        life_time: Option<LifeTime>,
        args: PosWithInfo,
    ) -> Result<(), runtime::Error> {
        self.scope_stacks
            .lock()
            .unwrap()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .vars
            .declare_var(name, value.to_owned(), line, type_, life_time);

        self.update_when(args, name, value)
    }

    pub fn get_var(&self, name: &str) -> Option<Variable> {
        self.scope_stacks
            .lock()
            .unwrap()
            .iter_mut()
            .find_map(|scope_stack| {
                scope_stack.iter_mut().rev().find_map(|scope| {
                    let var = &mut scope.vars;
                    var.validate_lifetime();
                    var.get_var(name).cloned()
                })
            })
    }

    pub fn set_var(
        &self,
        name: &str,
        args: PosWithInfo,
        postfix: &[AtomPostfix],
        value: Value,
        line: usize,
    ) -> Result<(), Error> {
        let mut scope_stacks = self.scope_stacks.lock().unwrap();
        let mut var_found = false;
        'outer_vars: for scope in scope_stacks.iter_mut() {
            let scopes_iter = scope.iter_mut().rev();

            for scope in scopes_iter {
                if scope.vars.set_var(name, args, postfix, &value)? {
                    var_found = true;
                    break 'outer_vars;
                }
            }
        }

        if var_found {
            // variable was set, update whens
            drop(scope_stacks);
            self.update_when(args, name, value)?;
            return Ok(());
        }

        // declare global
        scope_stacks
            .first_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .vars
            .declare_var(name, value.clone(), line, VarType::VarVar, None);

        drop(scope_stacks);
        self.update_when(args, name, value)?;

        Ok(())
    }

    /// Part of the function constructor
    /// - This declares a function and binds it to an object
    /// # Arg count
    /// - If you pass `None`, it can accept any number of arguments
    pub fn add_func(&self, func: FunctionVariant, arg_count: Option<usize>) -> ObjectRef {
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
            obj: Arc::downgrade(&obj),
        };
        self.funcs.lock().unwrap().0.push(state);
        obj
    }

    pub fn add_func_declare_var(
        &self,
        name: &str,
        func: FunctionVariant,
        arg_count: Option<usize>,
    ) {
        let line = if let FunctionVariant::FunctionDefined { body_line, .. } = func {
            body_line
        } else {
            0
        };
        let obj = self.add_func(func, arg_count);
        self.add_var(name, obj.into(), line, VarType::VarVar, None);
    }

    /// Tries to get the latest defined variable or function with the given name
    pub fn get_identifier(&self, name: &str, args: PosWithInfo) -> Option<DefineType> {
        // check functions first
        let func = self.get_func_info(name, args);
        let var = self.get_var(name);

        let Some(func) = func else {
            return var.map(DefineType::Var);
        };

        let Some(var) = var else {
            return Some(DefineType::Func(func));
        };

        // is the variable a function binding?
        if let Value::Object(Some(var)) = var.get_value() {
            if Weak::ptr_eq(&func.obj, &Arc::downgrade(var)) {
                return Some(DefineType::Func(func));
            }
        }

        let ret = match func.variant {
            FunctionVariant::FunctionDefined {
                body_line: defined_line,
                ..
            } => {
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

    pub fn push_when(&self, when: &When) {
        self.scope_stacks
            .lock()
            .unwrap()
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .whens
            .push(when.clone().into());
    }

    fn update_when(
        &self,
        eval_args: PosWithInfo,
        var_name: &str,
        value: Value,
    ) -> Result<(), Error> {
        // TODO: only update if when expr has a variable in it
        let when_stack = self
            .scope_stacks
            .lock()
            .unwrap()
            .iter()
            .flatten()
            .flat_map(|scope| scope.whens.iter().cloned())
            .collect::<Vec<_>>();

        for when in when_stack {
            when.eval_body(eval_args, var_name, value.to_owned())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DefineType {
    Var(Variable),
    Func(FunctionState),
}

#[derive(Debug, Default)]
pub struct ScopeState {
    vars: VariableState,
    // all of the `when` that are active
    whens: Vec<Arc<When>>,
}

#[derive(Debug, Default)]
pub struct VariableState(pub HashMap<String, Variable>);

impl VariableState {
    pub fn declare_var(
        &mut self,
        name: &str,
        value: Value,
        line: usize,
        type_: VarType,
        life_time: Option<LifeTime>,
    ) {
        self.0.insert(
            name.to_string(),
            Variable {
                previous: value.clone(),
                value,
                // TODO: whats the default?
                line,
                type_,
                life_time,
                create_time: if matches!(life_time, Some(LifeTime::Seconds(_))) {
                    Some(Instant::now())
                } else {
                    None
                },
            },
        );
    }

    /// Checks and removes any variables that has expired
    pub fn validate_lifetime(&mut self) {
        self.0.retain(|_, var| {
            let Some(LifeTime::Seconds(seconds)) = var.life_time else {
                return true;
            };

            let create_time = var.create_time.unwrap();
            let now = Instant::now();
            let duration = now.duration_since(create_time).as_secs_f64();
            duration < seconds
        });
    }

    pub fn get_var(&self, name: &str) -> Option<&Variable> {
        self.0.get(name)
    }

    pub fn set_var(
        &mut self,
        name: &str,
        args: PosWithInfo,
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
    previous: Value,
    line: usize,
    type_: VarType,
    life_time: Option<LifeTime>,
    create_time: Option<Instant>,
}

impl Variable {
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn get_previous_value(&self) -> &Value {
        &self.previous
    }

    pub fn set_value(
        &mut self,
        args: PosWithInfo,
        value: Value,
        postfix: &[AtomPostfix],
    ) -> Result<(), Error> {
        if postfix.is_empty() {
            // TODO: concrete error?
            if matches!(self.type_, VarType::VarConst | VarType::VarVar) {
                self.previous = self.value.clone();
                self.value = value;
            }
            return Ok(());
        }

        // below is for postfix operations, this only applies to const var and var var
        if !matches!(self.type_, VarType::ConstVar | VarType::VarVar) {
            // TODO: concrete error?
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

        // TODO: concrete error
        let Some(var) = var else {
            return Err(Error::Type("Cannot read properties of null".to_string()));
        };

        let mut var = var.lock().unwrap();

        self.previous = self.value.clone();
        // TODO: reuse code from postfix
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
pub struct Functions(pub Vec<FunctionState>);

#[derive(Debug, Clone)]
/// Function state
/// - A function is either
///   - binded to an object, which is the object that the function is called on
///   - it is a native function
pub struct FunctionState {
    pub arg_count: Option<usize>,
    variant: FunctionVariant,
    obj: Weak<Mutex<Object>>,
}

impl FunctionState {
    fn eval(&self, eval_args: PosWithInfo, args: Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error> {
        let interpreter = eval_args.extra.0;
        let state = &interpreter.state;

        match &self.variant {
            FunctionVariant::FunctionDefined {
                body,
                arg_names,
                body_line: _,
            } => {
                // gc should be done up to this point, so it should be safe
                let obj = self.obj.upgrade().unwrap();
                let mut obj = obj.lock().unwrap();

                obj.set_property("arguments", array::constructor(interpreter, args)?);
                let Value::Object(Some(args)) = obj.get_property("arguments").unwrap() else {
                    unreachable!();
                };
                let args = args.lock().unwrap();

                let pop_call_stack = || {
                    state.scope_stacks.lock().unwrap().pop();
                };

                state.scope_stacks.lock().unwrap().push(vec![ScopeState {
                    vars: VariableState::default(),
                    whens: Vec::new(),
                }]);

                // declare arguments
                for (arg_name, arg_value) in arg_names.iter().zip(args.array_obj_iter()) {
                    state.add_var(&arg_name.to_string(), arg_value, 0, VarType::VarVar, None);
                }

                let code_with_pos = Position::new_with_extra(body.as_str(), eval_args.extra);

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
                        let ret = match statement.eval(code_with_pos) {
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
        body_line: usize,
        /// Where the expression / scope is located as an index
        body: Arc<String>,
        /// Argument names
        arg_names: Arc<Vec<String>>,
    },
    Native(NativeFunc),
}

pub type NativeFunc = fn(&Interpreter, Vec<Wrapper<Cow<Value>>>) -> Result<Value, Error>;
