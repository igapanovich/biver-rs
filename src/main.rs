use crate::biver_result::{BiverError, BiverErrorSeverity, BiverResult, error, warning};
use crate::command_line_arguments::{Command, CommandLineArguments, ListCommand};
use crate::env::Env;
use crate::repository_data::RepositoryData;
use crate::repository_operations::{AmendResult, CheckOutResult, CommitResult, PreviewResult, RepositoryDataResult, RestoreResult, RewordResult, VersionResult};
use clap::Parser;
use colored::Colorize;
use std::io;
use std::process::ExitCode;

mod biver_result;
mod command_line_arguments;
mod env;
mod formatting;
mod hash;
mod image_magick;
mod known_file_types;
mod nickname;
mod repository_data;
mod repository_operations;
mod repository_paths;
mod version_id;
mod viewer;
mod xdelta3;

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    let env = Env {
        xdelta3_path: arguments.xdelta3_path,
        image_magick_path: arguments.image_magick_path,
    };

    match run_command(&env, arguments.command) {
        Ok(()) => ExitCode::SUCCESS,

        Err(BiverError {
            error_message,
            severity: BiverErrorSeverity::Warning,
        }) => {
            println!("{}", error_message.yellow());
            ExitCode::SUCCESS
        }

        Err(BiverError {
            error_message,
            severity: BiverErrorSeverity::Error,
        }) => {
            eprintln!("{}", error_message.red());
            ExitCode::FAILURE
        }
    }
}

fn run_command(env: &Env, command: Command) -> BiverResult<()> {
    match command {
        Command::Status { versioned_file_path, all } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            match repo_data {
                RepositoryDataResult::NotInitialized => println!("Not initialized"),
                RepositoryDataResult::Initialized(repository_data) => {
                    let has_uncommitted_changes = repository_operations::has_uncommitted_changes(&repo_paths, &repository_data)?;
                    formatting::print_repository_data(&repository_data, has_uncommitted_changes, all);
                }
            }

            success()
        }

        Command::List(ListCommand::Branches { versioned_file_path }) => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            formatting::print_branch_list(&repo_data);

            success()
        }

        Command::Preview { versioned_file_path, target } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            let version = match repository_operations::version(&repo_data, &target) {
                VersionResult::InvalidTarget => return error("Invalid target"),
                VersionResult::Ok(version) => version,
            };

            let preview_file_path = match repository_operations::preview(&repo_paths, version) {
                PreviewResult::NoPreviewAvailable => return error("No preview available"),
                PreviewResult::Ok(preview_file_path) => preview_file_path,
            };

            viewer::show_preview(&preview_file_path)?;

            Ok(())
        }

        Command::Compare {
            versioned_file_path,
            target1,
            target2,
        } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            let version_and_preview = |target: Option<&str>| {
                let version = match target {
                    None => repo_data.head_version(),
                    Some(target) => match repository_operations::version(&repo_data, target) {
                        VersionResult::InvalidTarget => return error(format!("Invalid target {}", target)),
                        VersionResult::Ok(version) => version,
                    },
                };

                match repository_operations::preview(&repo_paths, &version) {
                    PreviewResult::NoPreviewAvailable => error(format!("No preview available for {}", version.id.bs58())),
                    PreviewResult::Ok(preview) => Ok((version, preview)),
                }
            };

            let (version1, preview_file_path1) = version_and_preview(Some(&target1))?;
            let (version2, preview_file_path2) = version_and_preview(target2.as_deref())?;

            let formatted_versions = formatting::format_versions(&repo_data, &vec![version1, version2]);
            let description1 = &formatted_versions[0];
            let description2 = &formatted_versions[1];

            viewer::show_comparison(&preview_file_path1, description1, &preview_file_path2, description2)?;

            success()
        }

        Command::Commit {
            versioned_file_path,
            branch,
            description,
        } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?;

            let result = match repo_data {
                RepositoryDataResult::NotInitialized => repository_operations::commit_initial_version(env, &repo_paths, branch.as_deref(), description.as_deref())?,
                RepositoryDataResult::Initialized(mut repo_data) => {
                    repository_operations::commit_version(env, &repo_paths, &mut repo_data, branch.as_deref(), description.as_deref())?
                }
            };

            match result {
                CommitResult::Ok => success_ok(),
                CommitResult::NothingToCommit => warning("Nothing to commit"),
                CommitResult::BranchRequired => error("Branch required"),
                CommitResult::BranchAlreadyExists => error("Branch already exists"),
            }
        }

        Command::Amend {
            versioned_file_path,
            confirmed,
            description,
        } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let mut repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            if !confirmed {
                println!("Are you sure you want to overwrite the head version? (y/N)");
                let confirmed = read_yes_no_input()?.unwrap_or(false);
                if !confirmed {
                    return success();
                }
            }

            let result = repository_operations::amend_head(env, &repo_paths, &mut repo_data, description.as_deref())?;

            match result {
                AmendResult::Ok => success_ok(),
                AmendResult::NoUncommittedChanges => warning("No uncommitted changes"),
                AmendResult::HeadMustBeBranch => error("Head must be on a branch"),
                AmendResult::CannotAmendParent => error("Cannot amend head version because it has children"),
                AmendResult::HeadEqualsParent => error("Amend would result in head version file content being identical to its parent's file content. Use hard reset instead."),
            }
        }

        Command::Reword {
            versioned_file_path,
            target,
            description,
        } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let mut repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            let result = repository_operations::reword(&repo_paths, &mut repo_data, &target, &description)?;

            match result {
                RewordResult::Ok => success_ok(),
                RewordResult::InvalidTarget => error("Invalid target"),
            }
        }

        Command::Discard { versioned_file_path, confirmed } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            if !repository_operations::has_uncommitted_changes(&repo_paths, &repo_data)? {
                return warning("No uncommitted changes");
            }

            if !confirmed {
                println!("Are you sure you want to discard uncommitted changes? (y/N)");
                let confirmed = read_yes_no_input()?.unwrap_or(false);
                if !confirmed {
                    return success();
                }
            }

            repository_operations::discard(env, &repo_paths, &repo_data)?;

            success_ok()
        }

        Command::Checkout { versioned_file_path, target } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let mut repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            let result = repository_operations::check_out(env, &repo_paths, &mut repo_data, &target)?;

            match result {
                CheckOutResult::Ok => success_ok(),
                CheckOutResult::BlockedByUncommittedChanges => error("Cannot check out because there are uncommitted changes"),
                CheckOutResult::InvalidTarget => error("Invalid target"),
            }
        }

        Command::Restore {
            versioned_file_path,
            output,
            target,
        } => {
            let repo_paths = repository_operations::paths(versioned_file_path);
            let repo_data = repository_operations::data(&repo_paths)?.initialized()?;

            let result = repository_operations::restore(env, &repo_paths, &repo_data, &target, output.as_deref())?;

            match result {
                RestoreResult::Ok => success_ok(),
                RestoreResult::BlockedByUncommittedChanges => error("Cannot restore to the versioned file because there are uncommitted changes"),
                RestoreResult::InvalidTarget => error("Invalid target"),
            }
        }

        Command::Dependencies => {
            formatting::print_dependencies(xdelta3::ready(env), image_magick::ready(env));
            success()
        }
    }
}

fn success_ok() -> BiverResult<()> {
    println!("{}", "OK".green());
    Ok(())
}

fn success() -> BiverResult<()> {
    Ok(())
}

fn read_yes_no_input() -> BiverResult<Option<bool>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.eq_ignore_ascii_case("y") {
        return Ok(Some(true));
    }

    if input.eq_ignore_ascii_case("n") {
        return Ok(Some(false));
    }

    Ok(None)
}

trait RepositoryDataResultExtensions {
    fn initialized(self) -> BiverResult<RepositoryData>;
}

impl RepositoryDataResultExtensions for RepositoryDataResult {
    fn initialized(self) -> BiverResult<RepositoryData> {
        match self {
            RepositoryDataResult::NotInitialized => Err(BiverError {
                error_message: "Not initialized".to_string(),
                severity: BiverErrorSeverity::Error,
            }),
            RepositoryDataResult::Initialized(repository_data) => Ok(repository_data),
        }
    }
}
