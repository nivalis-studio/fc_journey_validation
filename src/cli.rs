use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub file_path: Option<PathBuf>,
}
