//! Contains structures that are uncertain until runtime

use super::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
/// Either a variable, or a value, or a function call
pub struct UncertainExpr<'a> {
    pub identifier: &'a str,
}

impl<'a> From<Pair<'a, Rule>> for UncertainExpr<'a> {
    fn from(value: pest::iterators::Pair<'a, super::Rule>) -> Self {
        let mut value = value.into_inner();

        let identifier = value.next().unwrap().as_str();

        Self { identifier }
    }
}
