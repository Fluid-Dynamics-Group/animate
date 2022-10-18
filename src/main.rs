mod cli;

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    Ok(())
}
