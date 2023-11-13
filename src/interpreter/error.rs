use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to evaluate: {0}")]
    EvalError(#[from] super::runtime::error::Error),
}
