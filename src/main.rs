use crate::cli_arguments::{CliArguments, Command, ListCommand};
use crate::repository_operations::{CheckOutResult, CommitResult, RepositoryDataResult};
use clap::Parser;
use colored::Colorize;
use std::io;
use std::process::ExitCode;

mod cli_arguments;
mod hash;
mod nickname;
mod print_utils;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;
mod xdelta3;

fn main() -> io::Result<ExitCode> {
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

            Ok(ExitCode::SUCCESS)
        }

        Command::List(list_commands) => match list_commands {
            ListCommand::Branches { versioned_file_path } => {
                let repo_paths = repository_operations::paths(versioned_file_path);
                let repo_data = repository_operations::data(&repo_paths)?;

                match repo_data {
                    RepositoryDataResult::NotInitialized => println!("Not initialized"),
                    RepositoryDataResult::Initialized(repository_data) => {
                        print_utils::print_branch_list(&repository_data);
                    }
                }

                Ok(ExitCode::SUCCESS)
            }
        },

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

        Command::Discard { versioned_file_path, confirmed } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let RepositoryDataResult::Initialized(repo_data) = repository_operations::data(&repo_paths)? else {
                return uninitialized();
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

        Command::Checkout { versioned_file_path, target } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let RepositoryDataResult::Initialized(mut repo_data) = repository_operations::data(&repo_paths)? else {
                return uninitialized();
            };

            let result = repository_operations::check_out(&repo_paths, &mut repo_data, &target)?;

            match result {
                CheckOutResult::Ok => {
                    println!("{}", "OK".green());
                    Ok(ExitCode::SUCCESS)
                }
                CheckOutResult::BlockedByUncommittedChanges => {
                    println!("{}", "Cannot check out because there are uncommitted changes".red());
                    Ok(ExitCode::FAILURE)
                }
                CheckOutResult::InvalidTarget => {
                    println!("{}", "Invalid target".red());
                    Ok(ExitCode::FAILURE)
                }
            }
        }

        Command::Dependencies => {
            let xdelta3_ready = xdelta3::ready();
            print_utils::print_dependencies(xdelta3_ready);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn uninitialized() -> io::Result<ExitCode> {
    println!("{}", "Not initialized".yellow());
    Ok(ExitCode::SUCCESS)
}
