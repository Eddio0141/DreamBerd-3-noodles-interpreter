//! Contains variable related structures

use pest::iterators::Pair;

use crate::interpreter::runtime::error::Error;
use crate::Interpreter;

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

impl<'a> VariableDecl<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.declare_var(self.name, value);

        Ok(())
    }
}

#[derive(Debug)]
pub struct VarSet<'a> {
    pub name: &'a str,
    pub expression: Expression<'a>,
}

impl<'a> From<Pair<'a, Rule>> for VarSet<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        // skip until identifier
        let mut value = value.into_inner();

        let name = value.next().unwrap().as_str();

        // skip until expression
        let mut value = value.skip_while(|pair| pair.as_rule() != Rule::expression);

        let expression = Expression::from(value.next().unwrap());

        Self { name, expression }
    }
}

impl<'a> VarSet<'a> {
    pub fn eval(&self, interpreter: &Interpreter<'a>) -> Result<(), Error> {
        let value = self.expression.eval(interpreter)?;
        interpreter.state.set_var(self.name, value).ok_or_else(|| {
            Error::VariableNotFound(format!("Variable {} not found", self.name.to_string()))
        })?;
        Ok(())
    }
}
