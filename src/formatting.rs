use crate::repository_data::{RepositoryData, Version};
use colored::{ColoredString, Colorize};

const MAX_VERSIONS_TO_PRINT: usize = 20;

pub fn print_repository_data(repo_data: &RepositoryData, has_uncommitted_changes: bool, all: bool) {
    let limit = if all { None } else { Some(MAX_VERSIONS_TO_PRINT) };

    let versions_to_print: Vec<_> = repo_data.iter_head_and_ancestors().collect();

    let prepared = prepared::prepare(repo_data, &versions_to_print, has_uncommitted_changes, limit);
    let prepared = colorization::colorize_prepared(&prepared);

    if let Some(off_screen_info) = &prepared.off_screen_info {
        println!("{}", off_screen_info);
    }

    for prepared_version in prepared.versions.iter().rev() {
        println!("{}", prepared_version);
    }

    if let Some(uncommitted_changes) = &prepared.uncommitted_changes {
        println!("{}", uncommitted_changes);
    }
}

pub fn format_versions(repo_data: &RepositoryData, versions: &[&Version]) -> Vec<String> {
    let prepared = prepared::prepare(repo_data, versions, false, None);
    prepared.versions.iter().map(|v| v.to_string()).collect()
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

mod colorization {
    use crate::formatting::prepared::{Prepared, PreparedOffScreen, PreparedUncommitedChanges, PreparedVersion};
    use colored::{ColoredString, Colorize};

    pub fn colorize_prepared(prepared: &Prepared<String>) -> Prepared<ColoredString> {
        Prepared {
            off_screen_info: prepared.off_screen_info.as_ref().map(colorize_off_screen_info),
            versions: prepared.versions.iter().map(colorize_version).collect(),
            uncommitted_changes: prepared.uncommitted_changes.as_ref().map(colorize_uncommitted_changes),
        }
    }

    pub fn colorize_off_screen_info(prepared_off_screen_info: &PreparedOffScreen<String>) -> PreparedOffScreen<ColoredString> {
        PreparedOffScreen {
            more_versions_text_offset: prepared_off_screen_info.more_versions_text_offset,
            more_versions_text: prepared_off_screen_info.more_versions_text.bright_black(),
            forking_branches_offset: prepared_off_screen_info.forking_branches_offset,
            forking_branches: prepared_off_screen_info.forking_branches.clone().map(|f| f.bright_cyan()),
        }
    }

    pub fn colorize_version(prepared_version: &PreparedVersion<String>) -> PreparedVersion<ColoredString> {
        PreparedVersion {
            creation_time: prepared_version.creation_time.blue(),
            creation_time_humanized: prepared_version.creation_time_humanized.bright_blue(),
            id: prepared_version.id.bright_black(),
            nickname: prepared_version.nickname.white(),
            head_badge: prepared_version.head_badge.clone().map(|h| h.magenta()),
            branch_badge: prepared_version.branch_badge.clone().map(|b| b.bright_cyan()),
            forking_branches: prepared_version.forking_branches.clone().map(|f| f.bright_cyan()),
            description: prepared_version.description.clone().map(|d| d.green()),
        }
    }

    pub fn colorize_uncommitted_changes(prepared_uncommitted_changes: &PreparedUncommitedChanges<String>) -> PreparedUncommitedChanges<ColoredString> {
        PreparedUncommitedChanges {
            uncommitted_changes_text: prepared_uncommitted_changes.uncommitted_changes_text.yellow(),
        }
    }
}

mod prepared {
    use crate::repository_data::{RepositoryData, Version};
    use crate::version_id::VersionId;
    use chrono_humanize::HumanTime;
    use std::collections::{HashMap, HashSet};
    use std::fmt;
    use std::fmt::{Display, Formatter};

    pub struct Prepared<T> {
        pub off_screen_info: Option<PreparedOffScreen<T>>,
        pub versions: Vec<PreparedVersion<T>>,
        pub uncommitted_changes: Option<PreparedUncommitedChanges<T>>,
    }

    pub struct PreparedOffScreen<T> {
        pub more_versions_text_offset: usize,
        pub more_versions_text: T,
        pub forking_branches_offset: usize,
        pub forking_branches: Option<T>,
    }

    impl<T: Display> Display for PreparedOffScreen<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{:>offset$}", "", offset = self.more_versions_text_offset)?;
            self.more_versions_text.fmt(f)?;

            if let Some(forking_branches) = &self.forking_branches {
                write!(f, "{:>offset$}", "", offset = self.forking_branches_offset)?;
                forking_branches.fmt(f)?;
            }

            Ok(())
        }
    }

    pub struct PreparedVersion<T> {
        pub creation_time: T,
        pub creation_time_humanized: T,
        pub id: T,
        pub nickname: T,
        pub head_badge: Option<T>,
        pub branch_badge: Option<T>,
        pub forking_branches: Option<T>,
        pub description: Option<T>,
    }

    impl<T: Display> Display for PreparedVersion<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            fn fmt_clearance(f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, " ")
            }

            self.creation_time.fmt(f)?;

            fmt_clearance(f)?;
            self.creation_time_humanized.fmt(f)?;

            fmt_clearance(f)?;
            self.id.fmt(f)?;

            fmt_clearance(f)?;
            self.nickname.fmt(f)?;

            if let Some(head_badge) = &self.head_badge {
                fmt_clearance(f)?;
                head_badge.fmt(f)?;
            }

            if let Some(branch_badge) = &self.branch_badge {
                fmt_clearance(f)?;
                branch_badge.fmt(f)?;
            }

            if let Some(forking_branches) = &self.forking_branches {
                fmt_clearance(f)?;
                forking_branches.fmt(f)?;
            }

            if let Some(description) = &self.description {
                fmt_clearance(f)?;
                description.fmt(f)?;
            }

            Ok(())
        }
    }

    pub struct PreparedUncommitedChanges<T> {
        pub uncommitted_changes_text: T,
    }

    impl<T: Display> Display for PreparedUncommitedChanges<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.uncommitted_changes_text.fmt(f)
        }
    }

    pub fn prepare(repo_data: &RepositoryData, versions_to_prepare: &[&Version], has_uncommitted_changes: bool, limit_from_end: Option<usize>) -> Prepared<String> {
        let mut prepared_versions = Vec::new();

        let head_version_ids: Vec<VersionId> = repo_data.iter_head_and_ancestors().map(|v| v.id).collect();

        let branches_forking_at_version_id: HashMap<VersionId, Vec<String>> = repo_data
            .branches
            .iter()
            .map(|(branch, branch_leaf_id)| {
                let join_version_id = repo_data
                    .iter_version_and_ancestors(*branch_leaf_id)
                    .map(|v| v.id)
                    .find(|id| head_version_ids.contains(id))
                    .unwrap();

                (join_version_id, branch.clone())
            })
            .fold(HashMap::new(), |mut acc, (version_id, branch)| {
                acc.entry(version_id).or_insert_with(Vec::new).push(branch);
                acc
            });

        let branches_by_version: HashMap<VersionId, String> = repo_data.branches.iter().map(|(branch, version_id)| (version_id.clone(), branch.clone())).collect();

        let limit_from_end = limit_from_end.unwrap_or(versions_to_prepare.len());
        let mut version_count = 0;
        let mut off_screen_branches = HashSet::new();

        let mut max_nickname_length = 0;
        let mut max_creation_time_humanized_length = 0;

        let head_branch = repo_data.head.branch();
        let head_version_id = repo_data.head_version().id;

        for version in versions_to_prepare.iter() {
            version_count += 1;

            let forking_branches = branches_forking_at_version_id.get(&version.id);
            let mut forking_branches = forking_branches.unwrap_or(&Vec::new()).clone();

            if version_count > limit_from_end {
                for b in forking_branches.iter() {
                    off_screen_branches.insert(b.clone());
                }
                continue;
            }

            let creation_time_local = version.creation_time.with_timezone(&chrono::Local);
            let creation_time_humanized = format!("({})", HumanTime::from(creation_time_local));

            let branch_on_version = branches_by_version.get(&version.id).cloned();

            if let Some(branch_on_version) = branch_on_version.clone() {
                forking_branches.retain(|b| b.ne(&branch_on_version));
            }
            let forking_branches = if forking_branches.len() > 0 {
                Some(format!("->[{}]", forking_branches.join(", ")))
            } else {
                None
            };

            let head_is_on_version = version.id == head_version_id;

            let head_badge = if head_is_on_version {
                if let Some(branch) = head_branch {
                    Some(format!("[HEAD = {}]", branch))
                } else {
                    Some("[HEAD]".to_string())
                }
            } else {
                None
            };

            let branch_badge = if let Some(branch) = &branch_on_version
                && branch_on_version.as_deref().ne(&head_branch)
            {
                Some(format!("[{}]", branch))
            } else {
                None
            };

            max_nickname_length = max_nickname_length.max(version.nickname.len());
            max_creation_time_humanized_length = max_creation_time_humanized_length.max(creation_time_humanized.len());

            prepared_versions.push(PreparedVersion {
                creation_time: creation_time_local.format("%Y-%m-%d %H:%M:%S").to_string(),
                creation_time_humanized: creation_time_humanized.to_string(),
                id: version.id.bs58(),
                nickname: version.nickname.clone(),
                head_badge,
                branch_badge,
                forking_branches,
                description: if version.description.len() > 0 { Some(version.description.to_string()) } else { None },
            });
        }

        for version in &mut prepared_versions {
            version.nickname = format!("{:>max_nickname_length$}", version.nickname);
            version.creation_time_humanized = format!("{:<max_creation_time_humanized_length$}", version.creation_time_humanized);
        }

        let total_version_count = versions_to_prepare.len();

        let off_screen_version_count = if total_version_count > limit_from_end {
            total_version_count - limit_from_end
        } else {
            0
        };

        let version_id_position = max_creation_time_humanized_length + 21;

        let off_screen_info = if off_screen_version_count == 0 {
            None
        } else {
            let more_versions_text = format!("...{} more versions", off_screen_version_count);

            let more_versions_slot_length = 23 + max_nickname_length;
            let forking_branches_offset = more_versions_slot_length - more_versions_text.len().min(more_versions_slot_length) + 1;

            let forking_branches = if off_screen_branches.len() == 0 {
                None
            } else {
                Some(format!("->[{}]", off_screen_branches.into_iter().collect::<Vec<_>>().join(", ")))
            };

            Some(PreparedOffScreen {
                more_versions_text_offset: version_id_position,
                more_versions_text,
                forking_branches_offset,
                forking_branches,
            })
        };

        let uncommitted_changes = if has_uncommitted_changes {
            Some(PreparedUncommitedChanges {
                uncommitted_changes_text: format!("{:<version_id_position$}(uncommitted changes)", ""),
            })
        } else {
            None
        };

        Prepared {
            off_screen_info,
            versions: prepared_versions,
            uncommitted_changes,
        }
    }
}
