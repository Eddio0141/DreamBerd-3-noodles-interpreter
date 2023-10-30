use super::Rule;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse code: {0}")]
    ParseError(#[from] pest::error::Error<Rule>),
}
