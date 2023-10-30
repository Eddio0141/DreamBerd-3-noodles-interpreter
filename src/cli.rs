use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use dreamberd_3_noodles_interpreter::interpreter::Interpreter;

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    /// The path to the script file to use
    file: PathBuf,
}

impl Cli {
    /// Process the CLI arguments and directly runs the interpreter
    pub fn process_from_cli(self) -> Result<()> {
        let file = fs::read_to_string(&self.file)
            .with_context(|| format!("Failed to read file at `{}`", self.file.display()))?;

        Interpreter::new_eval(&file)?;
        Ok(())
    }
}
