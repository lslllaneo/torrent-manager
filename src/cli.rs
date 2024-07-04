use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Debug, Parser, Serialize)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub source_dir: PathBuf,

    #[arg(short, long)]
    pub dest_dir: PathBuf,
}
