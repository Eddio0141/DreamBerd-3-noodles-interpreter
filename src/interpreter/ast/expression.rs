//! Contains expression related structures

use lazy_static::lazy_static;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::function::FunctionCall;
use super::runtime::value::Value;
use super::uncertain::UncertainExpr;
use super::Rule;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::logical_and, Assoc::Left)
            | Op::infix(Rule::logical_or, Assoc::Left)
            | Op::infix(Rule::comp_eq, Assoc::Left)
            | Op::infix(Rule::comp_ne, Assoc::Left)
            | Op::infix(Rule::comp_gt, Assoc::Left)
            | Op::infix(Rule::comp_ge, Assoc::Left)
            | Op::infix(Rule::comp_lt, Assoc::Left)
            | Op::infix(Rule::comp_le, Assoc::Left))
        .op(Op::prefix(Rule::expr_unary));
}

#[derive(Debug)]
/// Expression that can be evaluated
pub enum Expression<'a> {
    Atom(Atom<'a>),
    UnaryOperation {
        operator: UnaryOperator,
        right: Box<Expression<'a>>,
    },
    Operation {
        left: Box<Expression<'a>>,
        operator: Operator,
        right: Box<Expression<'a>>,
    },
}

impl<'a> From<Pair<'a, super::Rule>> for Expression<'a> {
    fn from(value: Pair<'a, super::Rule>) -> Self {
        let value = value.into_inner();

        PRATT_PARSER
            .map_primary(|primary| {
                if let Rule::expr_atom = primary.as_rule() {
                    Expression::Atom(primary.into())
                } else {
                    unreachable!()
                }
            })
            .map_infix(|left, op, right| {
                let operator = match op.as_rule() {
                    Rule::logical_and => Operator::And,
                    Rule::logical_or => Operator::Or,
                    Rule::comp_eq => Operator::Equal,
                    Rule::comp_ne => Operator::NotEqual,
                    Rule::comp_gt => Operator::GreaterThan,
                    Rule::comp_ge => Operator::GreaterThanOrEqual,
                    Rule::comp_lt => Operator::LessThan,
                    Rule::comp_le => Operator::LessThanOrEqual,
                    _ => unreachable!(),
                };

                Expression::Operation {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }
            })
            .map_prefix(|op, right| {
                let operator = match op.as_rule() {
                    Rule::expr_unary => UnaryOperator::Not,
                    _ => unreachable!(),
                };

                Expression::UnaryOperation {
                    operator,
                    right: Box::new(right),
                }
            })
            .parse(value)
    }
}

impl<'a> Expression<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        match self {
            Expression::Atom(atom) => dbg!(atom.eval(interpreter)),
            Expression::UnaryOperation { operator, right } => operator.eval(right, interpreter),
            Expression::Operation {
                left,
                operator,
                right,
            } => {
                let left = dbg!(left.eval(interpreter)?);
                let right = dbg!(right.eval(interpreter)?);

                let value = match operator {
                    Operator::Equal => Value::Boolean(left == right),
                    Operator::NotEqual => Value::Boolean(left != right),
                    Operator::GreaterThan => Value::Boolean(left > right),
                    Operator::GreaterThanOrEqual => Value::Boolean(left >= right),
                    Operator::LessThan => Value::Boolean(left < right),
                    Operator::LessThanOrEqual => Value::Boolean(left <= right),
                    Operator::And => Value::Boolean(left.into() && right.into()),
                    Operator::Or => Value::Boolean(left.into() || right.into()),
                };

                Ok(value)
            }
        }
    }
}

#[derive(Debug)]
pub enum Atom<'a> {
    UncertainExpr(UncertainExpr<'a>),
    FunctionCall(FunctionCall<'a>),
}

impl<'a> From<Pair<'a, Rule>> for Atom<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        let value = value.into_inner().next().unwrap();

        match value.as_rule() {
            Rule::var_or_value_or_func => Atom::UncertainExpr(value.into()),
            Rule::func_call => Atom::FunctionCall(value.into()),
            _ => unreachable!(),
        }
    }
}

impl<'a> Atom<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        match self {
            Atom::UncertainExpr(expr) => expr.eval(interpreter),
            Atom::FunctionCall(expr) => expr.eval(interpreter),
        }
    }
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,
}

impl<'a> UnaryOperator {
    pub fn eval(
        &self,
        right: &Expression<'a>,
        interpreter: &Interpreter<'a>,
    ) -> Result<Value, Error> {
        match self {
            UnaryOperator::Not => {
                let value = right.eval(interpreter)?;

                match self {
                    UnaryOperator::Not => Ok(!value),
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Operator {
    // comparison
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    // logical
    And,
    Or,
}
