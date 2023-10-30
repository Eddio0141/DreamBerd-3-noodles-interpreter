//! Contains structures that are uncertain until runtime

#[derive(Debug)]
/// Either a variable, or a value, or a function call
pub struct UncertainExpression<'a> {
    pub identifier: &'a str,
}
