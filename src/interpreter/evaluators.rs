use crate::{parsers::types::Position, Interpreter};

pub mod array;
pub mod expression;
pub mod function;
mod object;
pub mod parsers;
mod scope;
pub mod statement;
mod variable;

pub type EvalArgs<'a, 'b> = (&'a str, Position<'a, Interpreter<'b>>);

