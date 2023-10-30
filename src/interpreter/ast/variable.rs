//! Contains variable related structures

use super::expression::Expression;

#[derive(Debug)]
/// Declared variable
pub struct VariableDecl<'a> {
    pub name: &'a str,
    pub expression: Expression<'a>,
}
