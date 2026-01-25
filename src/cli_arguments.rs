use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show the current status of the repository
    #[command(alias = "st")]
    Status {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,

        /// Show all versions (by default, limited to 20 most recent)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// List commands
    #[command(subcommand)]
    List(ListCommand),

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

    /// Discard uncommitted changes
    Discard {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,
    },

    /// Check out a specific branch or version. If a version nickname is specified, the latest version with that nickname will be checked out.
    Checkout {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,

        /// Target branch or version to check out. May be one of the following (in order of precedence): branch name, version id, version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// List dependencies and check their statuses
    Dependencies,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List branches
    Branches {
        #[arg(short = 'f', long = "file", env = "BIVER_PATH")]
        versioned_file_path: PathBuf,
    },
}
