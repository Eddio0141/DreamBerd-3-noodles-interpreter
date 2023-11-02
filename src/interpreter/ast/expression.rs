//! Contains expression related structures

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;
use pest::iterators::Pair;

use super::function::FunctionCall;
use super::runtime::value::Value;
use super::uncertain::UncertainExpr;
use super::Rule;

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

impl<'a> Expression<'a> {
    fn atom_to_expression(value: Pair<'a, Rule>) -> Self {
        let mut value = value.into_inner().rev();
        let mut expr = Expression::Atom(value.next().unwrap().into());
        for pair in value {
            if pair.as_rule() == Rule::ws {
                continue;
            }

            expr = Expression::UnaryOperation {
                operator: pair.into(),
                right: Box::new(expr),
            };
        }

        expr
    }
}

impl<'a> From<Atom<'a>> for Expression<'a> {
    fn from(value: Atom<'a>) -> Self {
        Expression::Atom(value)
    }
}

impl<'a> From<Pair<'a, super::Rule>> for Expression<'a> {
    fn from(value: Pair<'a, super::Rule>) -> Self {
        let mut value = value.into_inner();
        let first = value.next().unwrap();

        // check quick return
        if value.peek().is_none() {
            return Expression::atom_to_expression(first);
        }

        // ws on the left and right of op needs to be added, and each op needs to have that info
        // atom -> (ws -> op -> ws) -> atom -> (ws -> op -> ws) -> atom
        // we then start creating the tree from left to the right
        // if the next op is lesser in ws, evaluate next op first
        // if the next op is equals in ws, do the usual ordering with operation order

        // ws count and op
        let mut priorities = Vec::new();
        // atoms
        let mut atoms = Vec::new();
        let mut last_ws = 0;
        let mut last_op = None;
        // let mut prev = None;
        for pair in value {
            match pair.as_rule() {
                Rule::ws => {
                    last_ws += pair.as_str().len();
                }
                Rule::expr_atom => {
                    priorities.push((last_ws, last_op.unwrap()));
                    atoms.push(pair);
                    last_ws = 0;
                }
                _ => {
                    last_op = Some(pair.into());
                }
            }
        }

        // work on expression
        let mut left = Expression::atom_to_expression(first);
        let mut left_pending = Vec::new();

        for (i, (ws, op)) in priorities.iter().enumerate() {
            let right = Expression::atom_to_expression(atoms.remove(0));
            let next_op = priorities.get(i + 1);

            // is the next op higher in priority?
            if let Some((next_ws, next_op)) = next_op {
                // check ws first, then operation type
                if (next_ws < ws) || (next_ws == ws && next_op > op) {
                    // beause we have to build from the right now, we need to store the left
                    // expr(left, op, right)
                    left_pending.insert(0, (left, op));
                    left = right;
                    continue;
                }
            }

            // now we need to do the pending ops
            // we need to drain left_pending
            left = Expression::Operation {
                left: Box::new(left),
                operator: *op,
                right: Box::new(right),
            };
            for (left_inner, op_inner) in left_pending.drain(..) {
                left = Expression::Operation {
                    left: Box::new(left_inner),
                    operator: *op_inner,
                    right: Box::new(left),
                };
            }
        }

        left
    }
}

impl<'a> Expression<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<Value, Error> {
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
                    Operator::NotEqual => Value::Boolean(left != right),
                    Operator::GreaterThan => Value::Boolean(left > right),
                    Operator::GreaterThanOrEqual => Value::Boolean(left >= right),
                    Operator::LessThan => Value::Boolean(left < right),
                    Operator::LessThanOrEqual => Value::Boolean(left <= right),
                    Operator::And => Value::Boolean(left.into() && right.into()),
                    Operator::Or => Value::Boolean(left.into() || right.into()),
                    Operator::Add => left + right,
                    Operator::Subtract => left - right,
                    Operator::Multiply => left * right,
                    Operator::Exponential => left.pow(right),
                    Operator::Divide => left / right,
                    Operator::Modulo => left % right,
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

impl<'a> From<Pair<'a, Rule>> for UnaryOperator {
    fn from(value: Pair<'a, Rule>) -> Self {
        match value.as_rule() {
            Rule::expr_unary => UnaryOperator::Not,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, Clone, Copy)]
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
    // arithmetic
    Add,
    Subtract,
    Multiply,
    Exponential,
    Divide,
    Modulo,
}

impl<'a> From<Pair<'a, Rule>> for Operator {
    fn from(value: Pair<'a, Rule>) -> Self {
        match value.as_rule() {
            Rule::comp_eq => Operator::Equal,
            Rule::comp_ne => Operator::NotEqual,
            Rule::comp_gt => Operator::GreaterThan,
            Rule::comp_ge => Operator::GreaterThanOrEqual,
            Rule::comp_lt => Operator::LessThan,
            Rule::comp_le => Operator::LessThanOrEqual,
            Rule::logical_and => Operator::And,
            Rule::logical_or => Operator::Or,
            Rule::math_add => Operator::Add,
            Rule::math_sub => Operator::Subtract,
            Rule::math_mul => Operator::Multiply,
            Rule::math_exp => Operator::Exponential,
            Rule::math_div => Operator::Divide,
            Rule::math_mod => Operator::Modulo,
            _ => unreachable!(),
        }
    }
}

impl From<Operator> for usize {
    fn from(value: Operator) -> Self {
        match value {
            Operator::Equal => 0usize,
            Operator::NotEqual => 0,
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
        usize::from(*self) == (*other).into()
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
