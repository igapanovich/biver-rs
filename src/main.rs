use crate::cli_arguments::{CliArguments, Commands};
use crate::repository_data::RepositoryData;
use crate::repository_operations::{CommitResult, RepositoryContextResult};
use clap::Parser;
use colored::Colorize;

mod cli_arguments;
mod hash;
mod nickname;
mod repository_context;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;

fn main() -> Result<(), std::io::Error> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Commands::Status { versioned_file_path } => {
            let repository_context = repository_operations::repository_context(&versioned_file_path)?;

            match repository_context {
                RepositoryContextResult::NotInitialized(_) => println!("Not initialized"),
                RepositoryContextResult::Initialized(repository_data) => print_repository_data(&repository_data.data),
            }

            Ok(())
        }
        Commands::Commit { versioned_file_path, description } => {
            let description = description.unwrap_or_default();
            let repository_context = repository_operations::repository_context(&versioned_file_path)?;

            let result = match repository_context {
                RepositoryContextResult::NotInitialized(repository_paths) => repository_operations::commit_initial_version(&repository_paths, &description)?,
                RepositoryContextResult::Initialized(repository_context) => repository_operations::commit_version(&repository_context, &description)?,
            };

            match result {
                CommitResult::Ok => println!("{}", "OK".green()),
                CommitResult::NothingToCommit => println!("{}", "Nothing to commit".yellow()),
            }

            Ok(())
        }
    }
}

fn print_repository_data(repository_data: &RepositoryData) {
    let mut sorted_versions = repository_data.versions.clone();
    sorted_versions.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));

    for version in &sorted_versions {
        println!(
            "{:<21}{:<12}{}",
            version.creation_time.format("%Y-%m-%d %H:%M:%S").to_string().blue(),
            version.nickname.white(),
            version.description.green()
        );
    }
}
