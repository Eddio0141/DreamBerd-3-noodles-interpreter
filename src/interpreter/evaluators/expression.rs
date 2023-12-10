//! Contains expression related structures

use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::{map_opt, value};
use nom::multi::{many0, many1};
use nom::sequence::{tuple, Tuple};
use nom::{character::complete::*, Parser};

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::parsers::types::Position;
use crate::parsers::{chunk, ws_count};
use crate::prelude::Wrapper;
use crate::Interpreter;

use super::function::FunctionCall;
use super::parsers::AstParseResult;

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

impl Expression {
    pub fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        // ws on the left and right of op needs to be added, and each op needs to have that info
        // atom -> (ws -> op -> ws) -> atom -> (ws -> op -> ws) -> atom
        // 1+ 2 * 3
        // we then start creating the tree from left to the right
        // if the next op is lesser in ws, evaluate next op first
        // if the next op is equals in ws, do the usual ordering with operation order

        // a chunk of (ws -> op -> ws) that has operation parsed and contains total ws
        let op_chunk =
            tuple((ws_count, Operator::parse, ws_count)).map(|(ws1, op, ws2)| (op, ws1 + ws2));

        let (input, (first_atom, priorities)) = ((
            Expression::atom_to_expression,
            many0(tuple((op_chunk, Expression::atom_to_expression))),
        ))
            .parse(input)?;

        // work on expression
        let (mut left, mut pending_unary) = first_atom;

        // handle initial unary
        let removed_unary = if let Some((op, ws)) = pending_unary.first() {
            if *ws == 0 {
                left = Expression::UnaryOperation {
                    operator: op[0],
                    right: Box::new(left),
                };
                true
            } else {
                false
            }
        } else {
            false
        };

        if removed_unary {
            pending_unary.remove(0);
        }

        dbg!(&left, &pending_unary);

        // if true, it will take from left_pending, if false it will take pending_unary
        let mut pending_order_is_left = pending_unary.iter().map(|_| false).collect::<Vec<_>>();
        let mut left_pending = Vec::new();

        let mut priorities = priorities.into_iter().peekable();
        while let Some(((op, ws), (right, right_pending_unary))) = priorities.next() {
            let next_op = priorities.peek();

            // is there next unary
            dbg!(&right_pending_unary);
            if !right_pending_unary.is_empty() {
                for right_pending_unary in right_pending_unary {
                    pending_unary.push(right_pending_unary);
                }
                left_pending.push((left, op));
                left = right;
                pending_order_is_left.extend(vec![false; pending_unary.len()]);
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
            for take_left in pending_order_is_left.drain(..) {
                if take_left {
                    let (left_inner, op_inner) = left_pending.pop().unwrap();
                    left = Expression::Operation {
                        left: Box::new(left_inner),
                        operator: op_inner,
                        right: Box::new(left),
                    };
                } else {
                    let (op_inner, _) = pending_unary.remove(0);
                    for operator in op_inner {
                        left = Expression::UnaryOperation {
                            operator,
                            right: Box::new(left),
                        };
                    }
                }
            }
        }

        Ok((input, dbg!(left)))
    }

    // fn apply_pending_unary_immediate TODO

    /// Parses atom with its unary operators
    /// - Returns the built expression and if any unprocessed unary operators
    ///     - Each item in the vector is a vector of unary operators
    ///     - Outer vector is meaning there's a ws between the unary operator groups
    /// - Order of the unary operators is from left to right
    fn atom_to_expression<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, (Self, Vec<(Vec<UnaryOperator>, usize)>)> {
        let (input, (unaries, expr)) = ((
            many0(tuple((many1(UnaryOperator::parse), ws_count))),
            Atom::parse,
        ))
            .parse(input)?;

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
                    Some((ret, ws_prior))
                }
            })
            .collect::<Vec<_>>();

        Ok((input, (Expression::Atom(expr), unaries)))
    }
}

impl From<Atom> for Expression {
    fn from(value: Atom) -> Self {
        Expression::Atom(value)
    }
}

impl Expression {
    pub fn eval(
        &self,
        interpreter: &Interpreter,
        code: &str,
    ) -> Result<Wrapper<Cow<Value>>, Error> {
        match self {
            Expression::Atom(atom) => atom.eval(interpreter, code),
            Expression::UnaryOperation { operator, right } => {
                operator.eval(right, interpreter, code)
            }
            Expression::Operation {
                left,
                operator,
                right,
            } => {
                let left = left.eval(interpreter, code)?;
                let right = right.eval(interpreter, code)?;

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
pub enum Atom {
    Value(Value),
    FunctionCall(FunctionCall),
}

impl Atom {
    pub fn eval(
        &self,
        interpreter: &Interpreter,
        code: &str,
    ) -> Result<Wrapper<Cow<Value>>, Error> {
        let value = match self {
            Atom::Value(value) => Cow::Borrowed(value),
            Atom::FunctionCall(expr) => Cow::Owned(expr.eval(interpreter, code)?),
        };

        Ok(Wrapper(value))
    }

    fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
        if let Ok((input, value)) = FunctionCall::parse(input) {
            return Ok((input, Atom::FunctionCall(value)));
        }

        // variable?
        if let Ok((input, var)) = map_opt(chunk::<_, ()>, |chunk: Position<_>| {
            input.extra.state.get_var(chunk.input)
        })(input)
        {
            return Ok((input, Atom::Value(var)));
        }

        // either an actual value or implicit string
        if let Ok((input, value)) = Value::parse(input) {
            return Ok((input, Atom::Value(value)));
        }

        // implicit string
        // take until `!`
        let (input, rest) = take_until("!")(input)?;

        Ok((input, Atom::Value(Value::String(rest.input.to_string()))))
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
        interpreter: &Interpreter,
        code: &str,
    ) -> Result<Wrapper<Cow<Value>>, Error> {
        let value = match self {
            UnaryOperator::Not => !right.eval(interpreter, code)?,
            UnaryOperator::Minus => (-right.eval(interpreter, code)?)?,
        };

        Ok(value)
    }

    fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
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
    fn parse<'a, 'b, 'c>(
        input: Position<'a, 'b, Interpreter<'c>>,
    ) -> AstParseResult<'a, 'b, 'c, Self> {
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
