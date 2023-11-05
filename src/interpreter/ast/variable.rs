//! Contains variable related structures

use pest::iterators::Pair;

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

use super::expression::Expression;
use super::Rule;

#[derive(Debug)]
/// Declared variable
pub struct VariableDecl {
    name: String,
    expression: Expression,
}

impl From<Pair<'_, Rule>> for VariableDecl {
    fn from(value: Pair<'_, Rule>) -> Self {
        // skip until identifier
        let mut value = value
            .into_inner()
            .skip_while(|pair| pair.as_rule() != Rule::identifier);

        let name = value.next().unwrap().as_str().to_string();

        // skip until expression
        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::expression);

        let expression = Expression::from(value.next().unwrap());

        Self { name, expression }
    }
}

impl VariableDecl {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.add_var(&self.name, value);

        Ok(())
    }
}

#[derive(Debug)]
pub struct VarSet {
    name: String,
    expression: Expression,
}

impl From<Pair<'_, Rule>> for VarSet {
    fn from(value: Pair<'_, Rule>) -> Self {
        // skip until identifier
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str().to_string();

        // skip until expression
        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::expression);

        let expression = Expression::from(value.next().unwrap());

        Self { name, expression }
    }
}

impl VarSet {
    pub fn eval(&self, interpreter: &Interpreter) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.set_var(&self.name, value);
        Ok(())
    }
}
