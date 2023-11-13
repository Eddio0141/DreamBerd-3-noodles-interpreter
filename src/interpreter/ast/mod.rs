use std::collections::{HashMap, LinkedList};

use nom::{branch::alt, combinator::map, multi::many0, IResult, InputLength};

use crate::{interpreter::static_analysis::Analysis, Interpreter};

use self::{
    expression::Expression,
    function::{FunctionCall, FunctionDef},
    variable::*,
};

use super::{
    runtime::{self, error::Error, state::InterpreterState, value::Value},
    static_analysis::{FunctionInfo, Scope},
};

mod expression;
pub mod function;
mod parsers;
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

    /// Parses code into an AST
    pub fn parse(code: &str) -> Self {
        if let Statement::ScopeBlock(ast) = Self::parse_scope(ParserInput {
            code,
            static_analysis: AnalysisProgress(Analysis::analyze(code).global_scope),
        }) {
            ast
        } else {
            unreachable!("parse_scope returned something other than a scope block")
        }
    }

    fn parse_scope(input: ParserInput) -> Statement {
        // TODO comment

        // first, we parse functions
        // TODO does this work in scopes
        let scope = input.static_analysis.0;

        let funcs = scope
            .functions
            .iter()
            .map(|func| FunctionDef::parse(input).unwrap().1)
            .collect();

        let (_, statements) = many0(Statement::parse)(input).unwrap();

        Statement::ScopeBlock(Self { funcs, statements })
    }
}

#[derive(Debug, Clone)]
/// Single statement that does something
pub enum Statement {
    FunctionCall(FunctionCall),
    ScopeBlock(Ast),
    VariableDecl(VariableDecl),
    VarSet(VarSet),
    Expression(Expression),
}

impl Statement {
    /// Evaluates the statement
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Option<Value>, runtime::error::Error> {
        match self {
            Statement::FunctionCall(function) => function.eval(interpreter).map(|_| None),
            Statement::VariableDecl(decl) => decl.eval(interpreter).map(|_| None),
            Statement::VarSet(var_set) => var_set.eval(interpreter).map(|_| None),
            Statement::ScopeBlock(ast) => ast.eval_scope(interpreter).map(|_| None),
            // does nothing
            Statement::Expression(_) => Ok(None),
        }
    }

    pub fn parse<'a>(input: ParserInput) -> IResult<ParserInput, Self> {
        alt((
            VariableDecl::parse,
            VarSet::parse,
            Ast::parse_scope,
            // those are fallback parsers
            FunctionCall::parse,
            Expression::parse,
        ))(input)
    }
}

#[derive(Debug, Clone)]
struct ParserInput<'a> {
    code: &'a str,
    static_analysis: AnalysisProgress<'a>,
}

impl<'a> InputLength for ParserInput<'a> {
    fn input_len(&self) -> usize {
        self.code.input_len()
    }
}

#[derive(Debug, Clone)]
struct AnalysisProgress<'a> {
    // global scope -> some scope -> some scope -> current scope
    scope_depth: Vec<Scope<'a>>,
    // N/A -> scope vec -> scope vec -> ...
    index_depth: Vec<usize>,
    // TODO store current funcs
}

impl AnalysisProgress<'_> {
    /// Gets all global functions and functions in the current scope
    pub fn current_funcs(&self) -> HashMap<&str, &FunctionInfo> {
        self.scope_depth
            .iter()
            .rev()
            .map(|scope| scope.functions.iter().map(|(k, v)| (*k, v)))
            .flatten()
            .collect()
    }
}
