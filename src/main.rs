use crate::cli_arguments::{CliArguments, Commands};
use crate::repository_context::RepositoryContext;
use crate::repository_data::Version;
use crate::repository_operations::{CommitResult, RepositoryContextResult};
use clap::Parser;
use colored::Colorize;
use std::io;
use std::process::ExitCode;

mod cli_arguments;
mod hash;
mod nickname;
mod repository_context;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;

fn main() -> io::Result<ExitCode> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Commands::Status { versioned_file_path, all } => {
            let repository_context = repository_operations::repository_context(&versioned_file_path)?;

            match repository_context {
                RepositoryContextResult::NotInitialized(_) => println!("Not initialized"),
                RepositoryContextResult::Initialized(repository_data) => print_repository_data(&repository_data, all)?,
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

const MAX_VERSIONS_TO_PRINT: usize = 20;

fn print_repository_data(repo: &RepositoryContext, all: bool) -> io::Result<()> {
    let mut current_version = repo.data.head_version();
    let mut printed_version_count = 0;
    let mut more_versions_off_screen = false;
    let mut versions_to_print: Vec<&Version> = Vec::new();

    loop {
        printed_version_count += 1;

        if printed_version_count > MAX_VERSIONS_TO_PRINT && !all {
            more_versions_off_screen = true;
            break;
        }

        versions_to_print.push(current_version);

        current_version = match current_version.parent {
            Some(parent) => repo.data.version(&parent).expect("The parent version must exist."),
            None => break,
        };
    }

    versions_to_print.reverse();

    if more_versions_off_screen {
        println!("...");
    }

    for version in &versions_to_print {
        let nickname_padding = nickname::max_length() + 1;

        let branch_badge = match repo.data.branch_on_version(&version.id) {
            Some(branch) => format!("[{}] ", branch),
            None => "".to_string(),
        };

        let head_badge = if repo.data.head_version().id == version.id { "[HEAD] " } else { "" };

        println!(
            "{:<21}{:<nickname_padding$}{}{}{}",
            version.creation_time.format("%Y-%m-%d %H:%M:%S").to_string().blue(),
            version.nickname.white(),
            branch_badge.bright_blue(),
            head_badge.magenta(),
            version.description.green()
        );
    }

    if repository_operations::has_uncommitted_changes(&repo)? {
        println!("{:<21}{}", "", "(uncommitted changes)".yellow());
    }

    Ok(())
}
