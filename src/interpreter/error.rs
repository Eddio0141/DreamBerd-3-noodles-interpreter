use super::Rule;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse code: {0}")]
    ParseError(#[from] Box<pest::error::Error<Rule>>),
    #[error("Failed to evaluate: {0}")]
    EvalError(#[from] super::runtime::error::Error),
}
