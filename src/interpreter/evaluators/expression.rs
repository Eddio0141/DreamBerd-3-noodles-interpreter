//! Contains expression related structures

use crate::interpreter::runtime::error::Error;
use crate::interpreter::runtime::value::Value;
use crate::parsers::types::Position;
use crate::Interpreter;

use super::function::FunctionCall;
use super::parsers::EvalResult;

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
    pub fn parse<'a>(code: Position<&Interpreter>) -> EvalResult<'a, Self> {
        todo!()
    }
}

/// Handles checking if the unary operator should be applied, and switching the apply flag
fn check_if_apply_unary(
    last_op: &mut Option<UnaryOperator>,
    current_op: &UnaryOperator,
    apply: &mut bool,
) -> bool {
    let mut ret = false;

    match last_op {
        Some(last_op) => {
            if last_op == current_op {
                *apply = !*apply;
            } else {
                if *apply {
                    ret = true;
                }
                // because operator is not the same (new op), which means it will apply
                *apply = true;
            }
        }
        None => {
            *apply = true;
        }
    }

    ret
}

impl From<Atom> for Expression {
    fn from(value: Atom) -> Self {
        Expression::Atom(value)
    }
}

impl Expression {
    pub fn eval<'a>(&'a self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        match self {
            Expression::Atom(atom) => atom.eval(interpreter),
            Expression::UnaryOperation { operator, right } => operator.eval(right, interpreter),
            Expression::Operation {
                left,
                operator,
                right,
            } => {
                let left = left.eval(interpreter)?;
                let right = right.eval(interpreter)?;

                let value = match operator {
                    Operator::Equal => Value::Boolean(left == right),
                    Operator::StrictEqual => Value::Boolean(left.strict_eq(&right)),
                    Operator::NotEqual => Value::Boolean(left != right),
                    Operator::StrictNotEqual => Value::Boolean(!left.strict_eq(&right)),
                    Operator::GreaterThan => Value::Boolean(left > right),
                    Operator::GreaterThanOrEqual => Value::Boolean(left >= right),
                    Operator::LessThan => Value::Boolean(left < right),
                    Operator::LessThanOrEqual => Value::Boolean(left <= right),
                    Operator::And => Value::Boolean(left.into() && right.into()),
                    Operator::Or => Value::Boolean(left.into() || right.into()),
                    Operator::Add => (left + right)?,
                    Operator::Subtract => (left - right)?,
                    Operator::Multiply => (left * right)?,
                    Operator::Exponential => left.pow(&right)?,
                    Operator::Divide => (left / right)?,
                    Operator::Modulo => (left % right)?,
                };

                Ok(value)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Atom {
    // UncertainExpr(UncertainExpr),
    // UncertainString(UncertainString),
    FunctionCall(FunctionCall),
}

impl Atom {
    pub fn eval<'a>(&'a self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        match self {
            // Atom::UncertainExpr(expr) => Ok(expr.eval(interpreter)),
            // Atom::UncertainString(expr) => Ok(expr.eval(interpreter)),
            Atom::FunctionCall(expr) => expr.eval(interpreter),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Minus,
}

impl UnaryOperator {
    pub fn eval<'a>(&'a self, right: &'a Expression, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
        let value = match self {
            UnaryOperator::Not => !right.eval(interpreter)?,
            UnaryOperator::Minus => (-right.eval(interpreter)?)?,
        };

        Ok(value)
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
        usize::from(*self) == usize::from(*self)
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        usize::from(*self).partial_cmp(&(*other).into())
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::from(*self).cmp(&(*other).into())
    }
}
