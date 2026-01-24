use crate::cli_arguments::{CliArguments, Commands};
use crate::repository_data::{RepositoryData, Version};
use crate::repository_operations::{CommitResult, RepositoryDataResult};
use clap::Parser;
use colored::Colorize;
use std::io;
use std::process::ExitCode;

mod cli_arguments;
mod hash;
mod nickname;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;

fn main() -> io::Result<ExitCode> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Commands::Status { versioned_file_path, all } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            match repo_data {
                RepositoryDataResult::NotInitialized => println!("Not initialized"),
                RepositoryDataResult::Initialized(repository_data) => {
                    let has_uncommitted_changes = repository_operations::has_uncommitted_changes(&repo_paths, &repository_data)?;
                    print_repository_data(&repository_data, has_uncommitted_changes, all);
                }
            }

            Ok(ExitCode::SUCCESS)
        }

        Commands::Commit {
            versioned_file_path,
            branch,
            description,
        } => {
            let description = description.unwrap_or_default();
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            let result = match repo_data {
                RepositoryDataResult::NotInitialized => repository_operations::commit_initial_version(&repo_paths, branch.as_deref(), &description)?,
                RepositoryDataResult::Initialized(mut repo_data) => repository_operations::commit_version(&repo_paths, &mut repo_data, branch.as_deref(), &description)?,
            };

            match result {
                CommitResult::Ok => {
                    println!("{}", "OK".green());
                    Ok(ExitCode::SUCCESS)
                }
                CommitResult::NothingToCommit => {
                    println!("{}", "Nothing to commit".yellow());
                    Ok(ExitCode::SUCCESS)
                }
                CommitResult::BranchRequired => {
                    println!("{}", "Branch required".red());
                    Ok(ExitCode::FAILURE)
                }
                CommitResult::BranchAlreadyExists => {
                    println!("{}", "Branch already exists".red());
                    Ok(ExitCode::FAILURE)
                }
            }
        }

        Commands::Discard { versioned_file_path, confirmed } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            let repo_data = match repo_data {
                RepositoryDataResult::NotInitialized => {
                    println!("{}", "Not initialized".yellow());
                    return Ok(ExitCode::SUCCESS);
                }
                RepositoryDataResult::Initialized(repo_data) => repo_data,
            };

            if !repository_operations::has_uncommitted_changes(&repo_paths, &repo_data)? {
                println!("{}", "No uncommitted changes".yellow());
                return Ok(ExitCode::SUCCESS);
            }

            if !confirmed {
                println!("Are you sure you want to discard uncommitted changes? (y/N)");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let confirmed = input.trim().eq_ignore_ascii_case("y");
                if !confirmed {
                    return Ok(ExitCode::SUCCESS);
                }
            }

            repository_operations::discard(&repo_paths, &repo_data)?;

            Ok(ExitCode::SUCCESS)
        }
    }
}

const MAX_VERSIONS_TO_PRINT: usize = 20;

fn print_repository_data(repo_data: &RepositoryData, has_uncommitted_changes: bool, all: bool) {
    let mut current_version = repo_data.head_version();
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
            Some(parent) => repo_data.version(&parent).expect("The parent version must exist."),
            None => break,
        };
    }

    versions_to_print.reverse();

    if more_versions_off_screen {
        println!("...");
    }

    for version in &versions_to_print {
        let nickname_padding = nickname::max_length() + 1;

        let branch_badge = match repo_data.branch_on_version(&version.id) {
            Some(branch) => format!("[{}] ", branch),
            None => "".to_string(),
        };

        let head_badge = if repo_data.head_version().id == version.id { "[HEAD] " } else { "" };

        println!(
            "{:<21}{:<nickname_padding$}{}{}{}",
            version.creation_time.format("%Y-%m-%d %H:%M:%S").to_string().blue(),
            version.nickname.white(),
            branch_badge.bright_blue(),
            head_badge.magenta(),
            version.description.green()
        );
    }

    if has_uncommitted_changes {
        println!("{:<21}{}", "", "(uncommitted changes)".yellow());
    }
}
