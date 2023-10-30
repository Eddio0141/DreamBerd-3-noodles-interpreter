pub mod error;

pub struct Interpreter;

impl Interpreter {
    /// Create a new interpreter and evaluate the given code
    /// - This is a synchronous function and will block until the code is finished executing
    pub fn new_eval(code: &str) -> Result<(), self::error::Error> {
        todo!()
    }
}
