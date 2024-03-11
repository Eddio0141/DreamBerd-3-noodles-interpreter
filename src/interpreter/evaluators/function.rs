//! Contains function related structures

use nom::{
    branch::alt, bytes::complete::*, character::complete::*, combinator::*, error::ErrorKind,
    sequence::*, *,
};

use crate::{
    interpreter::runtime::{error::Error, state::DefineType, value::Value},
    parsers::types::Position,
    runtime::state::FunctionState,
};
use crate::{parsers::*, Interpreter};

use super::{expression::Expression, variable::VarType, EvalArgs};
use super::{expression::FunctionExpr, parsers::AstParseResult};

#[derive(Debug, Clone)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall {
    name: String,
    args: Vec<Expression>,
}

impl FunctionCall {
    pub fn eval(&self, eval_args: EvalArgs) -> Result<Value, Error> {
        let interpreter = eval_args.1.extra;
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(arg.eval(eval_args)?);
        }

        interpreter.state.invoke_func(eval_args, &self.name, args)
    }

    fn try_get_func<'a, 'b, P, PO>(
        input: Position<'a, Interpreter>,
        identifier_term: P,
        fail_if_lower_identifier_order: bool,
    ) -> IResult<Position<'a, Interpreter>, (&'a str, FunctionState), ()>
    where
        P: Parser<Position<'a, Interpreter>, PO, ()>,
    {
        let mut identifier = identifier(identifier_term);

        let (input, identifier) = identifier(input)?;
        let identifier = identifier.into();

        let func = if fail_if_lower_identifier_order {
            if let Some(DefineType::Func(func)) = input.extra.state.get_identifier(identifier) {
                func
            } else {
                return Err(nom::Err::Error(()));
            }
        } else if let Some(func) = input.extra.state.get_func_info(identifier) {
            func
        } else {
            return Err(nom::Err::Error(()));
        };

        // does the function exist
        Ok((input, (identifier, func)))
    }

    pub fn parse_maybe_as_func<'a, 'b, P>(
        input: Position<'a, Interpreter>,
        identifier_term: Option<P>,
    ) -> AstParseResult<'a, Self>
    where
        P: Parser<Position<'a, Interpreter>, (), ()> + Clone,
    {
        Self::parse(input, identifier_term, true)
    }

    pub fn parse_as_func(input: Position<Interpreter>) -> AstParseResult<Self> {
        Self::parse::<fn(Position<Interpreter>) -> _>(input, None, false)
    }

    /// Parses a function call
    /// # Arguments
    /// - `fail_if_lower_identifier_order`: if true, this will fail the parser if an identifier is found that is a variable too
    fn parse<'a, 'b, P>(
        input: Position<'a, Interpreter>,
        identifier_term: Option<P>,
        fail_if_lower_identifier_order: bool,
    ) -> AstParseResult<'a, Self>
    where
        P: Parser<Position<'a, Interpreter>, (), ()> + Clone,
    {
        // function call syntax
        // - `func_name!`
        // with args
        // - `func_name arg1, arg2!`

        // try a stricter one first, and the relaxed after
        let strict_result = match identifier_term.clone() {
            Some(identifier_term) => Self::try_get_func(
                input,
                alt((identifier_term, char('!').map(|_| ()))),
                fail_if_lower_identifier_order,
            ),
            None => Self::try_get_func(input, char('!'), fail_if_lower_identifier_order),
        };
        let (input, (identifier, func)) = if let Ok(res) = strict_result {
            res
        } else {
            let relaxed_result = match identifier_term {
                Some(identifier_term) => {
                    Self::try_get_func(input, identifier_term, fail_if_lower_identifier_order)
                }
                None => Self::try_get_func(input, fail::<_, (), _>, fail_if_lower_identifier_order),
            };
            relaxed_result
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Fail)))?
        };

        // no args?
        let arg_count = func.arg_count.unwrap_or_default();
        if arg_count == 0 {
            // no args
            return Ok((
                input,
                Self {
                    name: identifier.to_string(),
                    args: Vec::new(),
                },
            ));
        }

        // has args
        let (input, _) = tuple((not(end_of_statement), ws))(input)?;

        let (mut input, mut args) = {
            let (input, (first_arg, _)) = tuple((Expression::parse, ws))(input)?;
            (input, vec![first_arg])
        };

        // grab arguments
        for _ in 0..arg_count - 1 {
            // TODO for expression, implement some way to either make the expression parse until the end of the statement or stringify the expression
            let (input_new, (_, _, expr, _)) =
                tuple((char(','), ws, Expression::parse, ws))(input)?;
            input = input_new;
            args.push(expr);
        }

        Ok((
            input,
            Self {
                name: identifier.to_string(),
                args,
            },
        ))
    }
}

#[derive(Debug, Clone)]
/// A function definition
pub struct FunctionDef {
    pub name: String,
    func: FunctionExpr,
}

const FUNCTION_HEADER: &[char] = &['f', 'u', 'n', 'c', 't', 'i', 'o', 'n'];

impl FunctionDef {
    pub fn parse(input: Position<Interpreter>) -> AstParseResult<Self> {
        // header
        let (input, first_ch) = satisfy(|c| !is_ws(c))(input)?;
        let header_start_index = FUNCTION_HEADER.iter().position(|c| *c == first_ch);
        let Some(header_start_index) = header_start_index else {
            return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
        };

        let (input, rest) = chunk(input)?;

        let mut function_header = FUNCTION_HEADER.iter().skip(header_start_index + 1);
        for ch in rest.input.chars() {
            loop {
                let function_ch = function_header.next();
                let Some(function_ch) = function_ch else {
                    return Err(Err::Error(nom::error::Error::new(input, ErrorKind::Fail)));
                };
                if *function_ch == ch {
                    break;
                }
            }
        }
        let arrow = || tag("=>");
        let identifier = identifier(arrow());

        let (input, (_, identifier, _)) = tuple((ws, identifier, ws))(input)?;

        let (input, expr) = FunctionExpr::parse(input)?;

        let instance = Self {
            name: identifier.input.to_string(),
            func: expr,
        };

        Ok((input, instance))
    }

    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let obj = self.func.eval(interpreter);
        let line = self.func.body_line;
        interpreter
            .state
            .add_var(&self.name, obj.into(), line, VarType::VarVar, None);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Return(Option<Expression>);

impl Return {
    pub fn parse(input: Position<Interpreter>) -> AstParseResult<Self> {
        let ret = tag("return");
        let empty_return = end_of_statement.map(|_| None);
        let expr_return = tuple((Expression::parse, end_of_statement)).map(|(expr, _)| Some(expr));
        let (input, (_, _, expr)) = tuple((ret, ws, alt((empty_return, expr_return))))(input)?;

        Ok((input, Self(expr)))
    }

    pub fn eval(&self, args: EvalArgs) -> Result<Option<Value>, Error> {
        self.0
            .as_ref()
            .map(|expr| expr.eval(args).map(|result| result.0.into_owned()))
            .transpose()
    }
}
