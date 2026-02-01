use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CommandLineArguments {
    /// Path to xdelta3 executable. If not specified, it will be searched in PATH.
    #[arg(global(true), long = "xdelta3-path", env = "BIVER_XDELTA3_PATH")]
    pub xdelta3_path: Option<PathBuf>,

    /// Path to ImageMagick executable. If not specified, it will be searched in PATH.
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

        /// Target branch or version to preview. May be one of the following (in order of precedence): branch name, version ID, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// Compare two versions using their previews
    #[command(alias = "cmp")]
    Compare {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to compare. May be one of the following (in order of precedence): branch name, version ID, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target1: String,

        /// (Default: head) Target branch or version to compare. May be one of the following (in order of precedence): branch name, version ID, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target2: Option<String>,
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

    /// Amend the head version
    Amend {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,

        /// New description
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,
    },

    /// Change description of the specified version
    Reword {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target version to reword. Must be a version ID.
        target: String,

        /// New description
        #[arg(value_name = "DESCRIPTION")]
        description: String,
    },

    /// Discard uncommitted changes
    Discard {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,
    },

    /// Reset the current branch back to the specified version
    Reset {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Also reset the versioned file
        #[arg(long = "hard")]
        hard: bool,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,

        /// Target version to reset to. Must be a version ID.
        target: String,
    },

    /// Check out a specific branch or version
    Checkout {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Target branch or version to preview. May be one of the following (in order of precedence): branch name, version ID, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// Set versioned file to the state it was in when the specified version was created
    Restore {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Output file path. If not specified, the versioned file path will be used.
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Target branch or version to restore. May be one of the following (in order of precedence): branch name, version ID, head offset (~, ~1, ~2), version nickname (adjective-noun, adjectivenoun, an).
        target: String,
    },

    /// Rename commands
    #[command(subcommand)]
    Rename(RenameCommand),

    /// Delete commands
    #[command(subcommand)]
    Delete(DeleteCommand),

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

#[derive(Subcommand)]
pub enum RenameCommand {
    /// Rename a branch
    Branch {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        #[arg(value_name = "OLD_NAME")]
        old_name: String,

        #[arg(value_name = "NEW_NAME")]
        new_name: String,
    },
}

#[derive(Subcommand)]
pub enum DeleteCommand {
    /// Delete a branch
    Branch {
        #[arg(short = 'f', long = "file", env = "BIVER_VERSIONED_FILE")]
        versioned_file_path: PathBuf,

        /// Do not ask for confirmation
        #[arg(short = 'y', long = "yes")]
        confirmed: bool,

        #[arg(value_name = "NAME")]
        name: String,
    },
}
