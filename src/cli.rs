use clap::Parser;

/// Animate pictures together into 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// print output of ffmpeg
    #[arg(short, long, default_value_t = false)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    Folder(Folder),
    Pattern(Pattern),
}

#[derive(Parser, Debug)]
struct Folder {
    path: PathBuf
}

#[derive(Parser, Debug)]
struct Pattern {
    path: PathBuf
}
