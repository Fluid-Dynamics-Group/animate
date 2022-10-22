use clap::Parser;
use clap::Subcommand;

use std::path::PathBuf;

/// Animate pictures together into videos with ffmpeg
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// print output of ffmpeg
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    Folder(Folder),
    Pattern(Pattern),
}

#[derive(Parser, Debug)]
pub(crate) struct Folder {
    path: PathBuf
}

#[derive(Parser, Debug)]
pub(crate) struct Pattern {
    path: Vec<PathBuf>
}
