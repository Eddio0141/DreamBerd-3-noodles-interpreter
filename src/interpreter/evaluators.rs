use crate::{parsers::types::Position, Interpreter};

pub mod expression;
pub mod function;
pub mod parsers;
mod scope;
pub mod statement;
mod variable;

pub type EvalArgs<'a, 'b, 'c, 'd> = (&'a str, Position<'b, 'c, Interpreter<'d>>);
