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

    pub(crate) framerate: usize,

    pub(crate) output_path: PathBuf,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// animate all files in a folder together
    Folder(Folder),
    /// animate a list of files (usually generated by xargs) together
    Pattern(Pattern),
}

#[derive(Parser, Debug)]
pub(crate) struct Folder {
    pub(crate) path: PathBuf
}

#[derive(Parser, Debug)]
pub(crate) struct Pattern {
    pub(crate) paths: Vec<PathBuf>
}
