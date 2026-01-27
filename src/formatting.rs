use crate::repository_data::{Head, RepositoryData, Version};
use chrono_humanize::HumanTime;
use colored::{ColoredString, Colorize};

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

    let FormattedVersionGroup {
        versions,
        humanized_creation_time_padding,
        nickname_padding,
    } = format_version_group(repo_data, &versions_to_print);

    if more_versions_off_screen {
        println!("...");
    }

    for formatted_version in &versions {
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

pub fn format_versions(repo_data: &RepositoryData, versions: &[&Version]) -> Vec<String> {
    let FormattedVersionGroup {
        versions,
        humanized_creation_time_padding,
        nickname_padding,
    } = format_version_group(repo_data, versions);

    let mut result = Vec::new();

    for formatted_version in versions {
        result.push(format!(
            "{} {:<humanized_creation_time_padding$} {} {:<nickname_padding$} {}{}{}",
            formatted_version.creation_time,
            formatted_version.creation_time_humanized,
            formatted_version.id,
            formatted_version.nickname,
            formatted_version.branch_badge,
            formatted_version.head_badge,
            formatted_version.description,
        ));
    }

    result
}

fn format_version_group(repo_data: &RepositoryData, versions: &[&Version]) -> FormattedVersionGroup {
    let mut formatted_versions = Vec::new();

    let mut humanized_creation_time_padding = 0;
    let mut nickname_padding = 0;

    let (head_version_id, head_branch) = match &repo_data.head {
        Head::Version(version_id) => (*version_id, None),
        Head::Branch(branch) => (repo_data.branches[branch], Some(branch)),
    };

    for version in versions {
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
        });
    }

    FormattedVersionGroup {
        versions: formatted_versions,
        humanized_creation_time_padding,
        nickname_padding,
    }
}

pub fn print_dependencies(xdelta3_ready: bool, image_magick_ready: bool) {
    fn optional_dep_status(ready: bool) -> ColoredString {
        if ready { "ready".green() } else { "not found".yellow() }
    }

    println!(
        "{:<14}{:<10}{}",
        "xdelta3",
        optional_dep_status(xdelta3_ready),
        "(Optional) Used for storing version file content as patches, which reduces repository size on disk"
    );
    println!(
        "{:<14}{:<10}{}",
        "ImageMagick",
        optional_dep_status(image_magick_ready),
        "(Optional) Used for creating version previews for image files"
    );
}

pub fn print_branch_list(repo_data: &RepositoryData) {
    for branch in repo_data.branches.keys() {
        println!("{}", branch)
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

struct FormattedVersionGroup {
    versions: Vec<FormattedVersion>,
    humanized_creation_time_padding: usize,
    nickname_padding: usize,
}
