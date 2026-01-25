use crate::repository_data::{RepositoryData, Version};
use chrono_humanize::HumanTime;
use colored::Colorize;

const MAX_VERSIONS_TO_PRINT: usize = 20;

pub fn print_repository_data(repo_data: &RepositoryData, has_uncommitted_changes: bool, all: bool) {
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

    let mut humanized_creation_time_padding = 0;
    let mut nickname_padding = 0;

    let mut formatted_versions = Vec::new();

    for version in &versions_to_print {
        let branch_badge = match repo_data.branch_on_version(&version.id) {
            Some(branch) => format!("[{}] ", branch),
            None => "".to_string(),
        };

        let head_badge = if repo_data.head_version().id == version.id { "[HEAD] " } else { "" };

        let creation_time_local = version.creation_time.with_timezone(&chrono::Local);
        let creation_time_humanized = HumanTime::from(creation_time_local);
        let creation_time_humanized = format!("({})", creation_time_humanized);

        humanized_creation_time_padding = humanized_creation_time_padding.max(creation_time_humanized.len());
        nickname_padding = nickname_padding.max(version.nickname.len());

        formatted_versions.push(FormattedVersion {
            creation_time: creation_time_local.format("%Y-%m-%d %H:%M:%S").to_string(),
            creation_time_humanized,
            id: version.id.bs58(),
            nickname: version.nickname.to_string(),
            branch_badge,
            head_badge: head_badge.to_string(),
            description: version.description.to_string(),
        })
    }

    if more_versions_off_screen {
        println!("...");
    }

    for formatted_version in &formatted_versions {
        println!(
            "{} {:<humanized_creation_time_padding$} {} {:<nickname_padding$} {}{}{}",
            formatted_version.creation_time.blue(),
            formatted_version.creation_time_humanized.bright_blue(),
            formatted_version.id.bright_black(),
            formatted_version.nickname.white(),
            formatted_version.branch_badge.bright_blue(),
            formatted_version.head_badge.magenta(),
            formatted_version.description.green(),
        );
    }

    if has_uncommitted_changes {
        let offset = 21 + humanized_creation_time_padding;
        println!("{:<offset$}{}", "", "(uncommitted changes)".yellow());
    }
}

struct FormattedVersion {
    creation_time: String,
    creation_time_humanized: String,
    id: String,
    nickname: String,
    branch_badge: String,
    head_badge: String,
    description: String,
}
