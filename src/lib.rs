//! # DreamBerd "noodles" interpreter
//!
//! This is an interpreter for the [DreamBerd](https://github.com/TodePond/DreamBerd) programming language
//!
//! # Example
//!
//! ## Simple use
//! ```
//! # use dreamberd_noodles_interpreter::interpreter::*;
//! let code = "print(12345)!";
//! Interpreter::new_eval(code).unwrap();
//! ```
//!
//! ## Advanced configurations!
//! ```
//! # use dreamberd_noodles_interpreter::interpreter::*;
//! let mut stdout = Vec::new();
//!
//! let code = r#"
//! var var var = 3!
//! print(var)!
//! var var = = 4!
//! print)var * =(!
//! "#;
//!
//! let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();
//! interpreter.eval(code).unwrap();
//!
//! let output_expected = br#"3
//! 12
//! "#;
//! ```

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub use interpreter::error::*;
pub use interpreter::*;

pub mod interpreter;
mod prelude;
#[cfg(test)]
mod tests;
