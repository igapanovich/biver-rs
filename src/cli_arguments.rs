use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show the current status of the repository
    #[command(alias = "st")]
    Status {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,
    },

    /// Commit current changes to a new version
    #[command(alias = "cm")]
    Commit {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,

        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
    },
}
