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
