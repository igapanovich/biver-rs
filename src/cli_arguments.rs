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

        /// Show all versions (by default, limited to 20 most recent)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// Commit current changes to a new version
    #[command(alias = "cm")]
    Commit {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,

        /// New branch to create
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        /// Description of the new version
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
    },
}
