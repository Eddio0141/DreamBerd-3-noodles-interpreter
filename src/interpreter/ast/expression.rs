//! Contains expression related structures

use pest::iterators::Pair;

use super::function::FunctionCall;
use super::uncertain::UncertainExpr;
use super::Rule;

#[derive(Debug)]
/// Expression that can be evaluated
pub enum Expression<'a> {
    Comparison(ComparisonExpr<'a>),
    Uncertain(UncertainExpr<'a>),
    FunctionCall(FunctionCall<'a>),
}

impl<'a> From<Pair<'a, super::Rule>> for Expression<'a> {
    fn from(value: Pair<'a, super::Rule>) -> Self {
        // go into inner
        let value = value.into_inner().next().unwrap();

        match value.as_rule() {
            Rule::comp_expr => Expression::Comparison(value.into()),
            Rule::var_or_value_or_func => Expression::Uncertain(value.into()),
            Rule::func_call => Expression::FunctionCall(value.into()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
/// Expression that compares expressions
pub struct ComparisonExpr<'a> {
    pub left: ComparableExpr<'a>,
    pub operator: ComparisonOperator,
    pub right: ComparableExpr<'a>,
}

impl<'a> From<Pair<'a, Rule>> for ComparisonExpr<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        let mut value = value.into_inner();

        let get_comp_expr = |pair: Pair<'a, _>| {
            let inner = pair.into_inner().next().unwrap();

            match inner.as_rule() {
                Rule::var_or_value_or_func => ComparableExpr::UncertainExpr(inner.into()),
                Rule::func_call => ComparableExpr::FunctionCall(inner.into()),
                _ => unreachable!(),
            }
        };

        let left = get_comp_expr(value.next().unwrap());

        // skip till operator
        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::comp_op);

        let operator = match value.next().unwrap().into_inner().next().unwrap().as_rule() {
            Rule::comp_eq => ComparisonOperator::Equal,
            Rule::comp_ne => ComparisonOperator::NotEqual,
            Rule::comp_gt => ComparisonOperator::GreaterThan,
            Rule::comp_ge => ComparisonOperator::GreaterThanOrEqual,
            Rule::comp_lt => ComparisonOperator::LessThan,
            Rule::comp_le => ComparisonOperator::LessThanOrEqual,
            _ => unreachable!(),
        };

        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::comp_expr_allowed);

        // get right
        let right = get_comp_expr(value.next().unwrap());

        Self {
            left,
            operator,
            right,
        }
    }
}

#[derive(Debug)]
/// Expressions that are allowed to be compared
pub enum ComparableExpr<'a> {
    UncertainExpr(UncertainExpr<'a>),
    FunctionCall(FunctionCall<'a>),
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
