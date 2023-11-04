use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Function {0} not found")]
    FunctionNotFound(String),
    #[error("Invalid argument count, expected {expected}, got {got}")]
    InvalidArgumentCount { expected: usize, got: usize },
    #[error("Runtime exception: {0}")]
    RuntimeException(String),
    #[error("TypeError: {0}")]
    Type(String),
}
