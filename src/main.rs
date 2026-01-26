use crate::biver_result::BiverResult;
use crate::cli_arguments::{CliArguments, Command, ListCommand};
use crate::repository_operations::{CheckOutResult, CommitResult, PreviewResult, RepositoryDataResult};
use clap::Parser;
use colored::Colorize;
use std::io;
use std::process::ExitCode;

mod biver_result;
mod cli_arguments;
mod hash;
mod image_magick;
mod known_file_types;
mod nickname;
mod preview;
mod print_utils;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;
mod xdelta3;

fn main() -> BiverResult<ExitCode> {
    let cli_arguments = CliArguments::parse();

    match cli_arguments.command {
        Command::Status { versioned_file_path, all } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            match repo_data {
                RepositoryDataResult::NotInitialized => println!("Not initialized"),
                RepositoryDataResult::Initialized(repository_data) => {
                    let has_uncommitted_changes = repository_operations::has_uncommitted_changes(&repo_paths, &repository_data)?;
                    print_utils::print_repository_data(&repository_data, has_uncommitted_changes, all);
                }
            }

            success("")
        }

        Command::List(ListCommand::Branches { versioned_file_path }) => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            match repo_data {
                RepositoryDataResult::NotInitialized => println!("Not initialized"),
                RepositoryDataResult::Initialized(repository_data) => {
                    print_utils::print_branch_list(&repository_data);
                }
            }

            success("")
        }

        Command::Preview { versioned_file_path, target } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let RepositoryDataResult::Initialized(repo_data) = repository_operations::data(&repo_paths)? else {
                return uninitialized();
            };

            let result = repository_operations::preview(&repo_paths, &repo_data, &target)?;

            match result {
                PreviewResult::Ok(preview_file_path) => match preview::open_window(preview_file_path) {
                    Ok(_) => success(""),
                    Err(_) => failure("Failed to open preview window"),
                },
                PreviewResult::NoPreviewAvailable => failure("No preview available"),
                PreviewResult::InvalidTarget => failure("Invalid target"),
            }
        }
            }
        }

        Command::Commit {
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
                CommitResult::Ok => success("OK"),
                CommitResult::NothingToCommit => warning("Nothing to commit"),
                CommitResult::BranchRequired => failure("Branch required"),
                CommitResult::BranchAlreadyExists => failure("Branch already exists"),
            }
        }

        Command::Discard { versioned_file_path, confirmed } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let RepositoryDataResult::Initialized(repo_data) = repository_operations::data(&repo_paths)? else {
                return uninitialized();
            };

            if !repository_operations::has_uncommitted_changes(&repo_paths, &repo_data)? {
                return warning("No uncommitted changes");
            }

            if !confirmed {
                println!("Are you sure you want to discard uncommitted changes? (y/N)");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let confirmed = input.trim().eq_ignore_ascii_case("y");
                if !confirmed {
                    return success("");
                }
            }

            repository_operations::discard(&repo_paths, &repo_data)?;

            success("")
        }

        Command::Checkout { versioned_file_path, target } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let RepositoryDataResult::Initialized(mut repo_data) = repository_operations::data(&repo_paths)? else {
                return uninitialized();
            };

            let result = repository_operations::check_out(&repo_paths, &mut repo_data, &target)?;

            match result {
                CheckOutResult::Ok => success("OK"),
                CheckOutResult::BlockedByUncommittedChanges => failure("Cannot check out because there are uncommitted changes"),
                CheckOutResult::InvalidTarget => failure("Invalid target"),
            }
        }

        Command::Dependencies => {
            let xdelta3_ready = xdelta3::ready();
            let image_magick_ready = image_magick::ready();
            print_utils::print_dependencies(xdelta3_ready, image_magick_ready);
            success("")
        }
    }
}

fn uninitialized() -> BiverResult<ExitCode> {
    println!("{}", "Not initialized".yellow());
    Ok(ExitCode::SUCCESS)
}

fn success(message: &str) -> BiverResult<ExitCode> {
    if !message.is_empty() {
        println!("{}", message.green());
    }
    Ok(ExitCode::SUCCESS)
}

fn warning(message: &str) -> BiverResult<ExitCode> {
    if !message.is_empty() {
        println!("{}", message.yellow());
    }
    Ok(ExitCode::SUCCESS)
}

fn failure(message: &str) -> BiverResult<ExitCode> {
    if !message.is_empty() {
        println!("{}", message.red());
    }
    Ok(ExitCode::FAILURE)
}
