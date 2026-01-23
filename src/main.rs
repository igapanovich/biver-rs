use crate::cli_arguments::{CliArguments, Commands};
use crate::repository_operations::CommitResult;
use clap::Parser;

mod cli_arguments;
mod hash;
mod repository_context;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;

fn main() -> Result<(), std::io::Error> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Commands::Commit { versioned_file_path, description } => {
            let description = description.unwrap_or_default();
            let repository_context = repository_operations::init_and_get_repository_context(&versioned_file_path)?;

            let result = repository_operations::commit_version(repository_context, description)?;

            match result {
                CommitResult::Ok => println!("OK"),
                CommitResult::NothingToCommit => println!("Nothing to commit"),
            }

            Ok(())
        }
    }
}
