use std::{
    fs,
    io::{BufRead, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use dreamberd_noodles_interpreter::{interpreter::Interpreter, InterpreterBuilder};

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    /// The path to the script file to use
    file: Option<PathBuf>,
}

impl Cli {
    /// Process the CLI arguments and directly runs the interpreter
    pub fn process_from_cli(self) -> Result<()> {
        if let Some(file) = self.file {
            let file = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read file at `{}`", file.display()))?;
            Interpreter::new_eval(&file)?;
            Ok(())
        } else {
            // repl mode
            let mut stdout = std::io::stdout().lock();
            let interpreter = InterpreterBuilder::with_stdout(&mut stdout).build();
            let mut stdout = std::io::stdout().lock();
            let mut stdin = std::io::stdin().lock();
            loop {
                let mut input = String::new();
                write!(stdout, ">>> ").context("Failed to write prompt")?;
                stdout.flush().context("Failed to flush stdout")?;
                stdin
                    .read_line(&mut input)
                    .context("Failed to read input for REPL")?;
                let res = interpreter.eval(&input);
                if let Err(err) = res {
                    eprintln!("{}", err);
                }
            }
        }
    }
}
