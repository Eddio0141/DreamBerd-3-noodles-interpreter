use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use dreamberd_noodles_interpreter::{
    interpreter::{runtime::value::Value, Interpreter},
    InterpreterBuilder,
};
use rustyline::{error::ReadlineError, DefaultEditor};

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

            let mut editor = DefaultEditor::new().context("Failed to start repl with history")?;

            loop {
                let line = editor.readline("> ");

                match line {
                    Ok(line) => {
                        // history isn't required but it's nice to have
                        let _ = editor.add_history_entry(&line);
                        let res = interpreter.eval_repl(&line);
                        match res {
                            Ok(mut res) => {
                                while let Some(Value::Undefined) = res.last() {
                                    res.pop();
                                }

                                if res.is_empty() {
                                    continue;
                                }

                                println!(
                                    "{}",
                                    res.iter()
                                        .map(|val| val.to_string())
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                );
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        }
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                        // ctrl-c | ctrl-d
                        break;
                    }
                    _ => (),
                }
            }

            Ok(())
        }
    }
}
