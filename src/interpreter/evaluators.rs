use crate::{parsers::types::Position, Interpreter};

pub mod array;
mod conditional;
pub mod expression;
pub mod function;
mod object;
pub mod parsers;
mod scope;
pub mod statement;
mod variable;

/// The arguments passed to the eval function
/// # Parameters
/// - The whole code
/// - Current position
pub type EvalArgs<'a> = (&'a str, Position<'a, Interpreter>);
