//! Contains expression related structures

use std::borrow::Cow;
use std::sync::Arc;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::*,
    combinator::*,
    error::ErrorKind,
    multi::{many0, many1, many_till},
    sequence::tuple,
    IResult, Parser,
};

use crate::{
    interpreter::{
        evaluators::statement::Statement,
        runtime::{error::Error, state::DefineType, value::Value},
    },
    parsers::{types::Position, *},
    prelude::Wrapper,
    runtime::{state::FunctionVariant, value::ObjectRef},
    Interpreter,
};

use super::{
    array::ArrayInitialiser, function::FunctionCall, object::ObjectInitialiser,
    parsers::AstParseResult,
};

#[derive(Debug, Clone)]
/// Expression that can be evaluated
pub enum Expression {
    Atom(Atom),
    UnaryOperation {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    Operation {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
}

type AtomToExpressionResult<'a> =
    AstParseResult<'a, (Expression, Vec<(Vec<UnaryOperator>, usize)>)>;

type NextExprOperation<'a> = Option<&'a (
    (Operator, usize),
    (Expression, Vec<(Vec<UnaryOperator>, usize)>),
)>;

impl Expression {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        Self::parser::<fn(PosWithInfo) -> _>(None)(input)
    }

    pub fn parser<'a, P>(
        implicit_string_term: Option<P>,
    ) -> impl Fn(PosWithInfo<'a>) -> AstParseResult<'a, Self>
    where
        P: Parser<PosWithInfo<'a>, PosWithInfo<'a>, ()> + Copy,
    {
        move |input| {
            // ws on the left and right of op needs to be added, and each op needs to have that info
            // atom -> (ws -> op -> ws) -> atom -> (ws -> op -> ws) -> atom
            // 1+ 2 * 3
            // we then start creating the tree from left to the right
            // if the next op is lesser in ws, evaluate next op first
            // if the next op is equals in ws, do the usual ordering with operation order

            // a chunk of (ws -> op -> ws) that has operation parsed and contains total ws
            let op_chunk =
                tuple((ws_count, Operator::parse, ws_count)).map(|(ws1, op, ws2)| (op, ws1 + ws2));

            let (input, (first_atom, priorities)) = tuple((
                Expression::atom_to_expression(implicit_string_term),
                many0(tuple((
                    op_chunk,
                    Expression::atom_to_expression(implicit_string_term),
                ))),
            ))(input)?;

            // work on expression
            let (mut left, mut pending_unary) = first_atom;

            // handle initial unary
            left =
                Self::apply_pending_unary_immediate(&mut pending_unary, left, priorities.first());

            // if true, it will take from left_pending, if false it will take pending_unary
            let mut pending_order_is_left = pending_unary.iter().map(|_| false).collect::<Vec<_>>();
            let mut left_pending = Vec::new();

            let mut priorities = priorities.into_iter().peekable();
            while let Some(((op, ws), (mut right, mut right_pending_unary))) = priorities.next() {
                let next_op = priorities.peek();

                right =
                    Self::apply_pending_unary_immediate(&mut right_pending_unary, right, next_op);

                // is there next unary
                if !right_pending_unary.is_empty() {
                    for right_pending_unary in right_pending_unary {
                        pending_unary.push(right_pending_unary);
                        pending_order_is_left.push(false);
                    }
                    left_pending.push((left, op));
                    left = right;
                    pending_order_is_left.push(true);
                    continue;
                }

                // is the next op higher in priority?
                if let Some(((next_op, next_ws), _)) = next_op {
                    // check ws first, then operation type
                    let next_ws = *next_ws;
                    let next_op = *next_op;
                    if (next_ws < ws) || (next_ws == ws && next_op > op) {
                        // beause we have to build from the right now, we need to store the left
                        // expr(left, op, right)
                        left_pending.push((left, op));
                        left = right;
                        pending_order_is_left.push(true);
                        continue;
                    }
                }

                // now we need to do the pending ops
                // we need to drain left_pending
                left = Expression::Operation {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
                let mut pending_order_removes = Vec::new();
                for (i, take_left) in pending_order_is_left.iter().enumerate() {
                    if *take_left {
                        let (left_inner, op_inner) = left_pending.pop().unwrap();
                        left = Expression::Operation {
                            left: Box::new(left_inner),
                            operator: op_inner,
                            right: Box::new(left),
                        };
                        pending_order_removes.push(i);
                    } else {
                        let (op_inner, ws) = pending_unary.last().unwrap();

                        // apply if unary ws is equals or less than next op ws
                        let apply_unary = match next_op {
                            Some(((_, next_ws), _)) => ws <= next_ws,
                            None => true,
                        };

                        if apply_unary {
                            for operator in op_inner {
                                left = Expression::UnaryOperation {
                                    operator: *operator,
                                    right: Box::new(left),
                                };
                            }

                            pending_order_removes.push(i);
                            pending_unary.pop();
                        }
                    }
                }

                // remove the pending orders
                for i in pending_order_removes.into_iter().rev() {
                    pending_order_is_left.remove(i);
                }
            }

            // apply the remaining pending unary
            for (op, _) in pending_unary.into_iter().rev() {
                for operator in op {
                    left = Expression::UnaryOperation {
                        operator,
                        right: Box::new(left),
                    };
                }
            }

            Ok((input, left))
        }
    }

    fn apply_pending_unary_immediate(
        pending_unary: &mut Vec<(Vec<UnaryOperator>, usize)>,
        mut left: Expression,
        next_op: NextExprOperation,
    ) -> Expression {
        while let Some((op, ws)) = pending_unary.first() {
            let apply = match next_op {
                Some(((_, next_ws), _)) => ws <= next_ws,
                None => *ws == 0,
            };

            if apply {
                for operator in op {
                    left = Expression::UnaryOperation {
                        operator: *operator,
                        right: Box::new(left),
                    };
                }

                pending_unary.remove(0);
            } else {
                break;
            }
        }

        left
    }

    /// Parses atom with its unary operators
    /// - Returns the built expression and if any unprocessed unary operators
    ///     - Each item in the vector is a vector of unary operators
    ///     - Outer vector is meaning there's a ws between the unary operator groups
    /// - Order of the unary operators is from left to right
    fn atom_to_expression<'a, P>(
        implicit_string_term: Option<P>,
    ) -> impl Fn(PosWithInfo<'a>) -> AtomToExpressionResult<'a>
    where
        P: Parser<PosWithInfo<'a>, PosWithInfo<'a>, ()> + Copy,
    {
        move |input| {
            let (input, (unaries, expr)) = tuple((
                many0(tuple((many1(UnaryOperator::parse), ws_count))),
                Atom::parser(implicit_string_term),
            ))(input)?;

            // 1. unaries must be reversed
            // 2. split by whitespace (its already done in the parser)
            // 3. on a group of same unary, only keep even number ones
            // 4. if cancelled out unary operations, combine the ws prior to them
            //    - e.g. `-  -- 1` -> `-   1` (combination of 1 ws and 2 ws)
            //    - if there's no more unary operations, then the ws tracking doesn't matter
            let mut ws_prior = 0;
            let unaries = unaries
                .into_iter()
                .rev()
                .filter_map(|(unaries, ws)| {
                    let ws = ws + ws_prior;
                    let mut unaries = unaries.iter();
                    let mut last_unary = unaries.next().unwrap();
                    let mut use_unary = true;
                    let mut ret = Vec::new();

                    for unary in unaries {
                        if unary == last_unary {
                            use_unary = !use_unary;
                        } else {
                            last_unary = unary;
                            if use_unary {
                                ret.push(*unary);
                            }
                            use_unary = true;
                        }
                    }

                    if use_unary {
                        ret.push(*last_unary);
                    }

                    if ret.is_empty() {
                        ws_prior = ws;
                        None
                    } else {
                        ws_prior = 0;
                        Some((ret, ws))
                    }
                })
                .collect::<Vec<_>>();

            Ok((input, (Expression::Atom(expr), unaries)))
        }
    }
}

impl From<Atom> for Expression {
    fn from(value: Atom) -> Self {
        Expression::Atom(value)
    }
}

impl Expression {
    pub fn eval(&self, args: PosWithInfo) -> Result<Wrapper<Cow<Value>>, Error> {
        match self {
            Expression::Atom(atom) => atom.eval(args),
            Expression::UnaryOperation { operator, right } => operator.eval(right, args),
            Expression::Operation {
                left,
                operator,
                right,
            } => {
                let left = left.eval(args)?;
                let right = right.eval(args)?;

                let value = match operator {
                    Operator::Equal => Value::Boolean(left.loose_eq(&right)?),
                    Operator::StrictEqual => Value::Boolean(left.strict_eq(&right)),
                    Operator::NotEqual => Value::Boolean(!left.loose_eq(&right)?),
                    Operator::StrictNotEqual => Value::Boolean(!left.strict_eq(&right)),
                    Operator::GreaterThan => Value::Boolean(matches!(
                        left.partial_cmp(&right),
                        Some(std::cmp::Ordering::Greater)
                    )),
                    Operator::GreaterThanOrEqual => Value::Boolean(
                        left.loose_eq(&right)?
                            || matches!(
                                left.partial_cmp(&right),
                                Some(std::cmp::Ordering::Greater)
                            ),
                    ),
                    Operator::LessThan => Value::Boolean(matches!(
                        left.partial_cmp(&right),
                        Some(std::cmp::Ordering::Less)
                    )),
                    Operator::LessThanOrEqual => Value::Boolean(
                        left.loose_eq(&right)?
                            || matches!(left.partial_cmp(&right), Some(std::cmp::Ordering::Less)),
                    ),
                    Operator::And => Value::Boolean(left.into() && right.into()),
                    Operator::Or => Value::Boolean(left.into() || right.into()),
                    Operator::Add => (left + right)?.0.into_owned(),
                    Operator::Subtract => (left - right)?.0.into_owned(),
                    Operator::Multiply => (left * right)?.0.into_owned(),
                    Operator::Exponential => left.pow(&right)?,
                    Operator::Divide => (left / right)?.0.into_owned(),
                    Operator::Modulo => (left % right)?.0.into_owned(),
                };

                Ok(Wrapper(Cow::Owned(value)))
            }
        }
    }
}

#[derive(Debug, Clone)]
// everything in here isn't evaluated until `eval`
pub struct Atom {
    pub value: AtomValue,
    postfix: Vec<AtomPostfix>,
}

#[derive(Debug, Clone)]
pub enum AtomValue {
    Value(Value),
    FunctionCall(FunctionCall),
    ObjectInitialiser(ObjectInitialiser),
    ArrayInitialiser(ArrayInitialiser),
    FunctionDef(FunctionExpr),
}

#[derive(Debug, Clone)]
pub struct FunctionExpr {
    arg_names: Vec<String>,
    body: String,
    pub body_line: usize,
}

impl From<&FunctionExpr> for FunctionVariant {
    fn from(func: &FunctionExpr) -> Self {
        Self::FunctionDefined {
            body_line: func.body_line,
            body: Arc::new(func.body.clone()),
            arg_names: Arc::new(func.arg_names.clone()),
        }
    }
}

impl FunctionExpr {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        // past header
        // func_args = { identifier ~ (comma ~ identifier)* }
        // ws_silent+ ~ identifier ~ (ws_silent+ ~ func_args? | ws_silent+) ~ arrow ~ ws_silent* ~ (scope_block | (expression ~ term))
        let comma = || char::<Position<_, _>, _>(',');
        let arg_identifier = || identifier(comma());
        let args = tuple((
            arg_identifier(),
            many0(
                tuple((ws, comma(), ws, arg_identifier())).map(|(_, _, _, identifier)| identifier),
            ),
        ))
        .map(|(first, mut rest)| {
            rest.insert(0, first);
            rest
        });
        let arrow = || tag("=>");
        let args = tuple((ws, args, ws, arrow())).map(|(_, args, _, _)| {
            args.into_iter()
                .map(|s| s.input.to_string())
                .collect::<Vec<_>>()
        });

        // this parses the function body
        // properly checks scope balance
        let scope = |input| {
            let scope_start = char('{');
            let (mut input, _) = scope_start(input)?;
            let scope_start = || tuple((ws, char('{'))).map(|_| Some(true));

            let scope_end = || tuple((ws, char('}'))).map(|_| Some(false));
            let mut statements_in_scope = many_till(
                Statement::parse,
                alt((scope_start(), scope_end(), eof.map(|_| None))),
            );

            let mut scope_track = 1usize;
            loop {
                if let Ok((i, (_, open_scope))) = statements_in_scope.parse(input) {
                    input = i;

                    if let Some(open_scope) = open_scope {
                        if open_scope {
                            scope_track = scope_track.checked_add(1).expect("scope overflow");
                        } else {
                            scope_track -= 1;
                            if scope_track == 0 {
                                return Ok((input, ()));
                            }
                        }

                        continue;
                    }
                }

                // this basically parses the rest of the code as this function's body, and this is fine
                // TODO parse the function as implicit string if it doesn't end with a scope?
                return Ok((input, ()));
            }
        };

        let (body, (args, _)) = tuple((alt((arrow().map(|_| Vec::new()), args)), ws))(input)?;

        let body_line = body.line;

        let (input, body) = alt((recognize(scope), recognize(Expression::parse)))(body)?;

        Ok((
            input,
            FunctionExpr {
                arg_names: args,
                body: body.to_string(),
                body_line,
            },
        ))
    }

    pub fn eval(&self, interpreter: &Interpreter) -> ObjectRef {
        interpreter
            .state
            .add_func(self.into(), Some(self.arg_names.len()))
    }
}

#[derive(Debug, Clone)]
pub enum AtomPostfix {
    DotNotation(String),
    BracketNotation(Expression),
}

impl AtomPostfix {
    pub fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        // object postfix can recurse
        // obj.postfix.postfix
        let obj_property = tuple((
            ws,
            char('.'),
            ws,
            alt((
                identifier(alt((char('!').map(|_| ()), AtomPostfix::parse.map(|_| ())))),
                terminated_chunk,
            )),
        ))
        .map(|(_, _, _, property)| AtomPostfix::DotNotation(property.to_string()));

        fn right_bracket(input: PosWithInfo) -> IResult<PosWithInfo, PosWithInfo, ()> {
            tag("]")(input)
        }

        // bracket notation https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Objects/Basics#bracket_notation
        // obj["postfix"]
        let obj_property_bracket = tuple((
            ws,
            char('['),
            ws,
            Expression::parser(Some(right_bracket)),
            ws,
            char(']'),
        ))
        .map(|(_, _, _, expr, _, _)| AtomPostfix::BracketNotation(expr));

        alt((obj_property, obj_property_bracket))(input)
    }

    pub fn parse_empty(input: PosWithInfo) -> IResult<PosWithInfo, (), ()> {
        match Self::parse(input) {
            Ok((input, _)) => Ok((input, ())),
            Err(_) => Err(nom::Err::Error(())),
        }
    }

    pub fn eval(&self, value: Cow<Value>, args: PosWithInfo) -> Result<Wrapper<Cow<Value>>, Error> {
        let Value::Object(obj) = value.into_owned() else {
            return Err(Error::Type("Cannot read properties".to_string()));
        };

        let Some(obj) = obj else {
            return Err(Error::Type("Cannot read properties of null".to_string()));
        };

        let obj = obj.lock().unwrap();

        match self {
            AtomPostfix::DotNotation(property) => {
                if let Some(value) = obj.get_property(property) {
                    return Ok(Wrapper(Cow::Owned(value.clone())));
                }
            }
            AtomPostfix::BracketNotation(expr) => {
                let value = expr.eval(args)?;
                let value = value.0.as_ref();

                match value {
                    Value::String(str) => {
                        if let Some(value) = obj.get_property(str) {
                            return Ok(Wrapper(Cow::Owned(value.clone())));
                        }
                    }
                    Value::Number(num) => {
                        // TODO: eventually handle floats, for now convert to int
                        let num = *num as i64;
                        if let Some(value) = obj.get_property(&num.to_string()) {
                            return Ok(Wrapper(Cow::Owned(value.clone())));
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(Wrapper(Cow::Owned(Value::Undefined)))
    }
}

impl Atom {
    pub fn eval(&self, args: PosWithInfo) -> Result<Wrapper<Cow<Value>>, Error> {
        let mut value = match &self.value {
            AtomValue::Value(value) => Cow::Borrowed(value),
            AtomValue::FunctionCall(expr) => Cow::Owned(expr.eval(args)?),
            AtomValue::ObjectInitialiser(expr) => Cow::Owned(expr.eval(args)?),
            AtomValue::ArrayInitialiser(expr) => Cow::Owned(expr.eval(args)?),
            AtomValue::FunctionDef(expr) => Cow::Owned(expr.eval(args.extra.0).into()),
        };

        for postfix in &self.postfix {
            value = postfix.eval(value, args)?.0;
        }

        Ok(Wrapper(value))
    }

    fn parser<'a, 'b: 'a, P>(
        implicit_string_term: Option<P>,
    ) -> impl Fn(PosWithInfo<'a>) -> AstParseResult<'a, Self>
    where
        P: Parser<PosWithInfo<'a>, PosWithInfo<'a>, ()> + Copy,
    {
        move |input| {
            // try parse without postfix and assume the whole thing is an identifier
            if let Ok((input, value)) = AtomValue::parse::<fn(PosWithInfo) -> _>(input, None) {
                return Ok((
                    input,
                    Atom {
                        value,
                        postfix: Vec::new(),
                    },
                ));
            }

            // try parse with postfix
            if let Ok((input, value)) = AtomValue::parse(input, Some(AtomPostfix::parse_empty)) {
                // has postfix, now grab them
                let (input, postfix) = many1(AtomPostfix::parse)(input)?;
                return Ok((input, Atom { value, postfix }));
            }

            // last resort, will return implicit string if all fails
            let (input, value) = AtomValue::parser_last_resort(implicit_string_term)(input);

            // ok, parse postfix too
            let (input, postfix) = many0(AtomPostfix::parse)(input)?;

            Ok((input, Atom { value, postfix }))
        }
    }
}

impl AtomValue {
    fn parse<'a, 'b, P>(
        input: PosWithInfo<'a>,
        postfix_separator: Option<P>,
    ) -> AstParseResult<'a, Self>
    where
        P: Parser<PosWithInfo<'a>, (), ()> + Clone,
    {
        if let Ok((input, value)) =
            FunctionCall::parse_maybe_as_func(input, postfix_separator.clone())
        {
            return Ok((input, AtomValue::FunctionCall(value)));
        }

        let variable_parse =
            |chunk: Position<_>| match input.extra.0.state.get_identifier(chunk.input, chunk) {
                Some(defined) => {
                    if let DefineType::Var(var) = defined {
                        Some(var)
                    } else {
                        None
                    }
                }
                None => None,
            };

        // variable?
        let variable_parse_result = match postfix_separator {
            Some(postfix_separator) => alt((
                map_opt(
                    identifier(alt((char('!').map(|_| ()), postfix_separator))),
                    variable_parse,
                ),
                map_opt(chunk, variable_parse),
            ))(input),
            None => alt((
                map_opt(terminated_chunk::<_, ()>, variable_parse),
                map_opt(chunk, variable_parse),
            ))(input),
        };
        if let Ok((input, var)) = variable_parse_result {
            // TODO function call
            return Ok((input, AtomValue::Value(var.get_value().clone())));
        }

        // func def
        if let Ok((input, expr)) = FunctionExpr::parse(input) {
            return Ok((input, AtomValue::FunctionDef(expr)));
        }

        Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Verify,
        )))
    }

    /// Parsing last resort
    fn parser_last_resort<'a, 'b: 'a, P>(
        implicit_string_term: Option<P>,
    ) -> impl FnMut(PosWithInfo<'a>) -> (PosWithInfo<'a>, Self)
    where
        P: Parser<PosWithInfo<'a>, PosWithInfo<'a>, ()> + Copy,
    {
        move |input| {
            // actual value?
            if let Ok((input, value)) = Value::parse(input) {
                return (input, AtomValue::Value(value));
            }

            // object initialiser
            // this isn't merged with `Value::parse` because object initialiser contains expressions, not values
            if let Ok((input, value)) = ObjectInitialiser::parse(input) {
                return (input, AtomValue::ObjectInitialiser(value));
            }

            // array initialiser
            if let Ok((input, value)) = ArrayInitialiser::parse(input) {
                return (input, AtomValue::ArrayInitialiser(value));
            }

            // implicit string
            // take until `!`
            let (input, str) = match implicit_string_term {
                Some(implicit_string_term) => alt((
                    take_until_parser(implicit_string_term),
                    take_until::<_, _, ()>("!"),
                    rest,
                ))(input)
                .unwrap(),
                None => alt((take_until::<_, _, ()>("!"), rest))(input).unwrap(),
            };

            (
                input,
                AtomValue::Value(Value::String(str.input.to_string())),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Minus,
}

impl UnaryOperator {
    pub fn eval<'a>(
        &'a self,
        right: &'a Expression,
        args: PosWithInfo,
    ) -> Result<Wrapper<Cow<Value>>, Error> {
        let value = match self {
            UnaryOperator::Not => !right.eval(args)?,
            UnaryOperator::Minus => (-right.eval(args)?)?,
        };

        Ok(value)
    }

    fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        alt((
            value(UnaryOperator::Not, char(';')),
            value(UnaryOperator::Minus, char('-')),
        ))(input)
    }
}

#[derive(Debug, Eq, Clone, Copy)]
pub enum Operator {
    // comparison
    Equal,
    StrictEqual,
    NotEqual,
    StrictNotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    // logical
    And,
    Or,
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Exponential,
    Divide,
    Modulo,
}

impl Operator {
    fn parse(input: PosWithInfo) -> AstParseResult<Self> {
        alt((
            value(Operator::StrictEqual, tag("===")),
            value(Operator::Equal, tag("==")),
            value(Operator::StrictNotEqual, tag(";==")),
            value(Operator::NotEqual, tag(";=")),
            value(Operator::GreaterThanOrEqual, tag(">=")),
            value(Operator::GreaterThan, char('>')),
            value(Operator::LessThanOrEqual, tag("<=")),
            value(Operator::LessThan, char('<')),
            value(Operator::And, tag("&&")),
            value(Operator::Or, tag("||")),
            value(Operator::Add, char('+')),
            value(Operator::Subtract, char('-')),
            value(Operator::Multiply, char('*')),
            value(Operator::Exponential, char('^')),
            value(Operator::Divide, char('/')),
            value(Operator::Modulo, char('%')),
        ))(input)
    }
}

impl From<Operator> for usize {
    fn from(value: Operator) -> Self {
        match value {
            Operator::Equal => 0,
            Operator::StrictEqual => 0,
            Operator::NotEqual => 0,
            Operator::StrictNotEqual => 0,
            Operator::GreaterThan => 0,
            Operator::GreaterThanOrEqual => 0,
            Operator::LessThan => 0,
            Operator::LessThanOrEqual => 0,
            Operator::And => 0,
            Operator::Or => 0,
            Operator::Add => 0,
            Operator::Subtract => 0,
            Operator::Multiply => 1,
            Operator::Exponential => 2,
            Operator::Divide => 1,
            Operator::Modulo => 1,
        }
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Self) -> bool {
        usize::from(*self) == usize::from(*other)
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::from(*self).cmp(&(*other).into())
    }
}
