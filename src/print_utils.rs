use crate::repository_data::{Head, RepositoryData, Version};
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
            Some(parent) => repo_data.version(parent).expect("The parent version must exist."),
            None => break,
        };
    }

    versions_to_print.reverse();

    let mut humanized_creation_time_padding = 0;
    let mut nickname_padding = 0;

    let mut formatted_versions = Vec::new();

    let (head_version_id, head_branch) = match &repo_data.head {
        Head::Version(version_id) => (*version_id, None),
        Head::Branch(branch) => (repo_data.branches[branch], Some(branch)),
    };

    for version in &versions_to_print {
        let version_branch = repo_data.branch_on_version(&version.id);

        let (branch_badge, head_badge) = match (version_branch, head_branch) {
            (Some(version_branch), Some(head_branch)) if version_branch == head_branch => {
                let branch_badge = String::from("");
                let head_badge = format!("[HEAD = {}] ", head_branch);
                (branch_badge, head_badge)
            }
            _ => {
                let branch_badge = match version_branch {
                    Some(branch) => format!("[{}] ", branch),
                    None => String::from(""),
                };
                let head_badge = if version.id == head_version_id { String::from("[HEAD] ") } else { String::from("") };
                (branch_badge, head_badge)
            }
        };

        let creation_time_local = version.creation_time.with_timezone(&chrono::Local);
        let creation_time_humanized = HumanTime::from(creation_time_local);
        let creation_time_humanized = format!("({})", creation_time_humanized);

        humanized_creation_time_padding = humanized_creation_time_padding.max(creation_time_humanized.len());
        nickname_padding = nickname_padding.max(version.nickname.len());

        formatted_versions.push(FormattedVersion {
            creation_time: creation_time_local.format("%Y-%m-%d %H:%M:%S").to_string(),
            creation_time_humanized,
            id: version.id.bs58(),
            nickname: version.nickname.clone(),
            branch_badge,
            head_badge,
            description: version.description.clone(),
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

pub fn print_dependencies(xdelta3_ready: bool) {
    let xdelta3_status = if xdelta3_ready { "ready".green() } else { "not found".yellow() };
    println!(
        "xdelta3: {:<10} (Optional) Used for storing version file content as patches, which reduces repository size on disk",
        xdelta3_status
    );
}

pub fn print_branch_list(repo_data: &RepositoryData) {
    for branch in repo_data.branches.keys() {
        println!("{}", branch)
    }
}
