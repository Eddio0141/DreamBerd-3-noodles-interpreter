//! Contains variable related structures

use pest::iterators::Pair;

use super::expression::Expression;
use super::Rule;

#[derive(Debug)]
/// Declared variable
pub struct VariableDecl<'a> {
    pub name: &'a str,
    pub expression: Expression<'a>,
}

impl<'a> From<Pair<'a, Rule>> for VariableDecl<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        // skip until identifier
        let mut value = value
            .into_inner()
            .skip_while(|pair| pair.as_rule() != Rule::identifier);

        let name = value.next().unwrap().as_str();

        // skip until expression
        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::expression);

        let expression = Expression::from(value.next().unwrap());

        Self { name, expression }
    }
}
