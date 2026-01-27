use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CliArguments {
    #[arg(global(true), long = "xdelta3-path", env = "BIVER_XDELTA3_PATH")]
    pub xdelta3_path: Option<PathBuf>,

    #[arg(global(true), long = "image-magick-path", env = "BIVER_IMAGE_MAGICK_PATH")]
    pub image_magick_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show the current status of the repository
    #[command(alias = "st")]
    Status {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Show all versions (by default, limited to 20 most recent)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// List commands
    #[command(subcommand)]
    List(ListCommand),

    /// Preview a version
    #[command(alias = "pv")]
    Preview {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to preview. May be one of the following (in order of precedence): branch name, version id, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// Compare two versions using their previews
    #[command(alias = "cmp")]
    Compare {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to compare. May be one of the following (in order of precedence): branch name, version id, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target1: String,

        /// Target branch or version to compare. May be one of the following (in order of precedence): branch name, version id, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target2: String,
    },

    /// Commit current changes to a new version
    #[command(alias = "ct")]
    Commit {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
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
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,
    },

    /// Check out a specific branch or version
    Checkout {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to preview. May be one of the following (in order of precedence): branch name, version id, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// Set versioned file to the state it was in when the specified version was created
    Apply {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to apply. May be one of the following (in order of precedence): branch name, version id, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// List dependencies and check their statuses
    Dependencies,
}

#[derive(Subcommand)]
pub enum ListCommand {
    /// List branches
    Branches {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,
    },
}
