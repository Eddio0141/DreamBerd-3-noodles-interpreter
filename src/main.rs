use anyhow::Result;
use clap::Parser;
use cli::Cli;

mod cli;

fn main() -> Result<()> {
    Cli::parse().process_from_cli()
}
