use crate::cli_arguments::{CliArguments, Commands};
use crate::repository_data::RepositoryData;
use crate::repository_operations::{CommitResult, RepositoryContextResult};
use clap::Parser;
use colored::Colorize;
use std::process::ExitCode;

mod cli_arguments;
mod hash;
mod nickname;
mod repository_context;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;

fn main() -> Result<ExitCode, std::io::Error> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Commands::Status { versioned_file_path } => {
            let repository_context = repository_operations::repository_context(&versioned_file_path)?;

            match repository_context {
                RepositoryContextResult::NotInitialized(_) => println!("Not initialized"),
                RepositoryContextResult::Initialized(repository_data) => print_repository_data(&repository_data.data),
            }
        }

        Commands::Commit {
            versioned_file_path,
            branch,
            description,
        } => {
            let description = description.unwrap_or_default();
            let repository_context = repository_operations::repository_context(&versioned_file_path)?;

            let result = match repository_context {
                RepositoryContextResult::NotInitialized(repository_paths) => repository_operations::commit_initial_version(&repository_paths, branch.as_deref(), &description)?,
                RepositoryContextResult::Initialized(repository_context) => repository_operations::commit_version(&repository_context, branch.as_deref(), &description)?,
            };

            match result {
                CommitResult::Ok => println!("{}", "OK".green()),
                CommitResult::NothingToCommit => println!("{}", "Nothing to commit".yellow()),
                CommitResult::BranchRequired => {
                    println!("{}", "Branch required".red());
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_repository_data(repository_data: &RepositoryData) {
    let mut sorted_versions = repository_data.versions.clone();
    sorted_versions.sort_by(|a, b| b.creation_time.cmp(&a.creation_time).reverse());

    for version in &sorted_versions {
        let nickname_padding = nickname::max_length() + 1;

        let branch_badge = match repository_data.branch_on_version(&version.id) {
            Some(branch) => format!("[{}] ", branch),
            None => "".to_string(),
        };

        let head_badge = if repository_data.head_version().id == version.id { "[HEAD] " } else { "" };

        println!(
            "{:<21}{:<nickname_padding$}{}{}{}",
            version.creation_time.format("%Y-%m-%d %H:%M:%S").to_string().blue(),
            version.nickname.white(),
            branch_badge.bright_blue(),
            head_badge.magenta(),
            version.description.green()
        );
    }
}
