//! Contains expression related structures

use super::uncertain::UncertainExpression;

#[derive(Debug)]
/// Expression that can be evaluated
pub enum Expression<'a> {
    ComparisonExpression(ComparisonExpression<'a>),
    UncertainExpression(UncertainExpression<'a>),
}

#[derive(Debug)]
/// Expression that compares expressions
pub struct ComparisonExpression<'a> {
    pub left: UncertainExpression<'a>,
    pub operator: ComparisonOperator,
    pub right: UncertainExpression<'a>,
}

#[derive(Debug)]
/// Comparison operator
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
