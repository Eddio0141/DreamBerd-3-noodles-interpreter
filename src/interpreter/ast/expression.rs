//! Contains expression related structures

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;
use pest::iterators::Pair;

use super::function::FunctionCall;
use super::runtime::value::Value;
use super::uncertain::*;
use super::Rule;

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
    /// Convert a pair into an expression
    /// - Returns the built expression and if any unprocessed unary operators
    ///     - Each item in the vector is a vector of unary operators
    ///     - Outer vector is meaning there's a ws between the unary operator groups
    /// - Order of the unary operators is from left to right
    fn atom_to_expression(value: Pair<'_, Rule>) -> (Self, Vec<Vec<UnaryOperator>>) {
        let mut value = value.into_inner().rev();
        let mut expr = Expression::Atom(value.next().unwrap().into());
        let mut last_op = None;
        let mut apply = false;

        let mut cut_off = false;
        // number of ws before the unary operator
        let mut cut_off_apply = false;
        let mut cut_off_last_op = None;
        let mut unary_ops = Vec::new();
        let mut current_unary_ops = Vec::new();
        // !;!!;
        // ! !!;
        // because unary operator also doesn't use brackets to show precedence, we need to cut it off by the ws
        for pair in value {
            if pair.as_rule() == Rule::ws {
                if cut_off_apply {
                    current_unary_ops.push(cut_off_last_op.unwrap());
                    cut_off_apply = false;
                    cut_off_last_op = None;
                }

                if !current_unary_ops.is_empty() {
                    unary_ops.push(current_unary_ops.clone());
                    current_unary_ops.clear();
                }

                cut_off = true;
                continue;
            }

            if cut_off {
                // currently is unary operator
                let op = pair.into();
                if check_if_apply_unary(&mut cut_off_last_op, &op, &mut cut_off_apply) {
                    current_unary_ops.push(cut_off_last_op.unwrap());
                }

                cut_off_last_op = Some(op);
                continue;
            }

            let op = pair.into();
            if check_if_apply_unary(&mut last_op, &op, &mut apply) {
                expr = Expression::UnaryOperation {
                    operator: last_op.unwrap(),
                    right: Box::new(expr),
                };
            }
            last_op = Some(op);
        }

        // one last adding
        if apply {
            expr = Expression::UnaryOperation {
                operator: last_op.unwrap(),
                right: Box::new(expr),
            };
        }

        // add cut off
        if cut_off_apply {
            current_unary_ops.push(cut_off_last_op.unwrap());
        }

        if !current_unary_ops.is_empty() {
            unary_ops.push(current_unary_ops);
        }

        (expr, unary_ops)
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

impl From<Pair<'_, super::Rule>> for Expression {
    fn from(value: Pair<'_, super::Rule>) -> Self {
        let mut value = value.into_inner();
        let first = value.next().unwrap();

        // check quick return
        if value.peek().is_none() {
            let (mut expr, ops) = Expression::atom_to_expression(first);
            for op in ops.into_iter().flatten() {
                expr = Expression::UnaryOperation {
                    operator: op,
                    right: Box::new(expr),
                };
            }
            return expr;
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
        let (mut left, mut pending_unary) = Expression::atom_to_expression(first);
        // if true, it will take from left_pending, if false it will take pending_unary
        let mut pending_order_is_left = pending_unary.iter().map(|_| false).collect::<Vec<_>>();
        let mut left_pending = Vec::new();

        for (i, (ws, op)) in priorities.iter().enumerate() {
            let (right, mut right_pending_unary) = Expression::atom_to_expression(atoms.remove(0));
            let next_op = priorities.get(i + 1);

            // is there next unary
            if !right_pending_unary.is_empty() {
                pending_unary.append(&mut right_pending_unary);
                left_pending.push((left, op));
                left = right;
                pending_order_is_left.extend(vec![false; pending_unary.len()]);
                pending_order_is_left.push(true);
                continue;
            }

            // is the next op higher in priority?
            if let Some((next_ws, next_op)) = next_op {
                // check ws first, then operation type
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
                operator: *op,
                right: Box::new(right),
            };
            for take_left in pending_order_is_left.drain(..) {
                if take_left {
                    let (left_inner, op_inner) = left_pending.pop().unwrap();
                    left = Expression::Operation {
                        left: Box::new(left_inner),
                        operator: *op_inner,
                        right: Box::new(left),
                    };
                } else {
                    let op_inner = pending_unary.remove(0);
                    for operator in op_inner {
                        left = Expression::UnaryOperation {
                            operator,
                            right: Box::new(left),
                        };
                    }
                }
            }
        }

        left
    }
}

impl Expression {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
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
    UncertainExpr(UncertainExpr),
    UncertainString(UncertainString),
    FunctionCall(FunctionCall),
}

impl From<Pair<'_, Rule>> for Atom {
    fn from(value: Pair<'_, Rule>) -> Self {
        match value.as_rule() {
            Rule::var_or_value_or_func => Atom::UncertainExpr(value.into()),
            Rule::var_or_string_or_func => Atom::UncertainString(value.into()),
            Rule::func_call => Atom::FunctionCall(value.into()),
            _ => unreachable!(),
        }
    }
}

impl Atom {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<Value, Error> {
        match self {
            Atom::UncertainExpr(expr) => Ok(expr.eval(interpreter)),
            Atom::UncertainString(expr) => Ok(expr.eval(interpreter)),
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
    pub fn eval(&self, right: &Expression, interpreter: &Interpreter) -> Result<Value, Error> {
        let value = match self {
            UnaryOperator::Not => !right.eval(interpreter)?,
            UnaryOperator::Minus => (-right.eval(interpreter)?)?,
        };

        Ok(value)
    }
}

impl From<Pair<'_, Rule>> for UnaryOperator {
    fn from(value: Pair<'_, Rule>) -> Self {
        let value = value.into_inner().next().unwrap();
        match value.as_rule() {
            Rule::logical_unary_not => UnaryOperator::Not,
            Rule::math_unary_minus => UnaryOperator::Minus,
            _ => unreachable!(),
        }
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

impl From<Pair<'_, Rule>> for Operator {
    fn from(value: Pair<'_, Rule>) -> Self {
        match value.as_rule() {
            Rule::comp_eq => Operator::Equal,
            Rule::comp_strict_eq => Operator::StrictEqual,
            Rule::comp_ne => Operator::NotEqual,
            Rule::comp_strict_ne => Operator::StrictNotEqual,
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
