use std::{
    collections::HashMap,
    ops::RangeFrom,
    str::{CharIndices, Chars},
};

use nom::{
    branch::alt, combinator::map, Compare, IResult, InputIter, InputLength, InputTake, Needed,
    Slice, UnspecializedInput,
};

use crate::{interpreter::static_analysis::Analysis, Interpreter};

use self::{
    expression::Expression,
    function::{FunctionCall, FunctionDef},
    variable::*,
};

use super::{
    runtime::{self, error::Error, state::InterpreterState, value::Value},
    static_analysis::{FunctionInfo, ScopeInfo},
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
        if let (_, Statement::ScopeBlock(ast)) = Self::parse_scope(ParserInput {
            code,
            static_analysis: &AnalysisProgress::new(Analysis::analyze(code).global_scope),
        })
        .unwrap()
        {
            ast
        } else {
            unreachable!("parse_scope returned something other than a scope block")
        }
    }

    fn parse_scope(input: ParserInput) -> IResult<ParserInput, Statement> {
        // TODO comment

        // first, we parse functions
        // TODO does this work in scopes
        let scope = input.static_analysis.scope_depth.last().unwrap();

        // let funcs = scope
        //     .functions
        //     .iter()
        //     .map(|func| FunctionDef::parse(input).unwrap().1)
        //     .collect();

        // let (_, statements) = many0(Statement::parse)(input).unwrap();

        // Statement::ScopeBlock(Self { funcs, statements })

        todo!()
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
            map(FunctionCall::parse, |func| Statement::FunctionCall(func)),
            Expression::parse,
        ))(input)
    }
}

impl From<Statement> for Expression {
    /// Extracts the expression variant from the statement
    /// # Panics
    /// - If the statement is not an expression
    fn from(value: Statement) -> Self {
        let Statement::Expression(expression) = value else {
            panic!("Statement is not an expression")
        };

        expression
    }
}

#[derive(Debug, Clone)]
struct AnalysisProgress<'a> {
    // global scope -> some scope -> some scope -> current scope
    scope_depth: Vec<ScopeInfo<'a>>,
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

    pub fn new(scope: ScopeInfo) -> Self {
        Self {
            scope_depth: vec![scope],
            index_depth: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ParserInput<'a> {
    code: &'a str,
    static_analysis: &'a AnalysisProgress<'a>,
}

impl<'a> InputLength for ParserInput<'a> {
    fn input_len(&self) -> usize {
        self.code.input_len()
    }
}

impl InputTake for ParserInput<'_> {
    fn take(&self, count: usize) -> Self {
        let code = self.code.take(count);
        Self {
            code,
            static_analysis: &self.static_analysis,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (left, right) = self.code.take_split(count);
        (
            Self {
                code: left,
                static_analysis: &self.static_analysis,
            },
            Self {
                code: right,
                static_analysis: &self.static_analysis,
            },
        )
    }
}

impl<'a> InputIter for ParserInput<'a> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;

    fn iter_indices(&self) -> Self::Iter {
        self.code.char_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.code.chars()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        for (o, c) in self.code.char_indices() {
            if predicate(c) {
                return Some(o);
            }
        }
        None
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        let mut cnt = 0;
        for (index, _) in self.code.char_indices() {
            if cnt == count {
                return Ok(index);
            }
            cnt += 1;
        }
        if cnt == count {
            return Ok(self.code.len());
        }
        Err(Needed::Unknown)
    }
}

impl<'a> Compare<&'a str> for ParserInput<'a> {
    fn compare(&self, t: &'a str) -> nom::CompareResult {
        self.code.compare(t)
    }

    fn compare_no_case(&self, t: &'a str) -> nom::CompareResult {
        self.code.compare_no_case(t)
    }
}

impl UnspecializedInput for ParserInput<'_> {}

impl Slice<RangeFrom<usize>> for ParserInput<'_> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        Self {
            code: self.code.slice(range),
            static_analysis: self.static_analysis,
        }
    }
}

impl<'a> From<ParserInput<'a>> for &'a str {
    fn from(input: ParserInput<'a>) -> Self {
        input.code
    }
}
