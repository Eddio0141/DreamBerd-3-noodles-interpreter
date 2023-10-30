//! Contains function related structures

use super::expression::Expression;

#[derive(Debug)]
/// A function call that is 100% certain its a function call
pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: Vec<Expression<'a>>,
}
