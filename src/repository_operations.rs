use crate::repository_data::{ContentBlobKind, Head, RepositoryData, Version};
use crate::repository_paths::RepositoryPaths;
use crate::version_id::VersionId;
use crate::{hash, image_magick, known_file_types, nickname, xdelta3};
use chrono::Utc;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

const DEFAULT_BRANCH: &str = "main";

const MAX_CONSECUTIVE_PATCHES: usize = 7;

pub fn paths(versioned_file_path: PathBuf) -> RepositoryPaths {
    let extension = match versioned_file_path.extension() {
        Some(extension) => {
            let mut extension = OsString::from(extension);
            extension.push(".biver");
            extension
        }
        None => OsString::from("biver"),
    };

    let repository_dir_path = versioned_file_path.with_extension(extension);

    let data_file_path = repository_dir_path.join("data.json");

    RepositoryPaths {
        versioned_file: versioned_file_path,
        repository_dir: repository_dir_path,
        data_file: data_file_path,
    }
}

pub enum RepositoryDataResult {
    Initialized(RepositoryData),
    NotInitialized,
}

pub fn data(repository_paths: &RepositoryPaths) -> io::Result<RepositoryDataResult> {
    if !repository_paths.data_file.exists() {
        return Ok(RepositoryDataResult::NotInitialized);
    }

    let data_file_contents = fs::read(&repository_paths.data_file)?;
    let repository_data = serde_json::from_slice(&data_file_contents)?;

    Ok(RepositoryDataResult::Initialized(repository_data))
}

pub enum CommitResult {
    Ok,
    NothingToCommit,
    BranchRequired,
    BranchAlreadyExists,
}

pub fn commit_initial_version(repo_paths: &RepositoryPaths, new_branch: Option<&str>, description: &str) -> io::Result<CommitResult> {
    if !fs::exists(&repo_paths.repository_dir)? {
        fs::create_dir(&repo_paths.repository_dir)?;
    } else if fs::exists(&repo_paths.data_file)? {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "The data file already exists."));
    }

    commit_version_common(repo_paths, None, new_branch, description)
}

pub fn commit_version(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, new_branch: Option<&str>, description: &str) -> io::Result<CommitResult> {
    commit_version_common(repo_paths, Some(repo_data), new_branch, description)
}

fn commit_version_common(repo_paths: &RepositoryPaths, repo_data: Option<&mut RepositoryData>, new_branch: Option<&str>, description: &str) -> io::Result<CommitResult> {
    let versioned_file = File::open(&repo_paths.versioned_file)?;

    let xxh3_128 = hash::xxh3_128(&versioned_file)?;

    if let Some(repo_data) = &repo_data
        && repo_data.head_version().versioned_file_xxh3_128 == xxh3_128
    {
        return Ok(CommitResult::NothingToCommit);
    }

    let branch = match new_branch {
        Some(new_branch) => {
            if let Some(repo_data) = &repo_data
                && repo_data.branches.contains_key(new_branch)
            {
                return Ok(CommitResult::BranchAlreadyExists);
            }
            new_branch.to_string()
        }
        None => match repo_data.as_ref() {
            None => DEFAULT_BRANCH.to_string(),
            Some(RepositoryData { head: Head::Branch(branch), .. }) => branch.to_string(),
            Some(RepositoryData { head: Head::Version(_), .. }) => return Ok(CommitResult::BranchRequired),
        },
    };

    let (content_blob_kind, base_content_blob_file_name) = if !xdelta3::ready() {
        (ContentBlobKind::Full, "")
    } else {
        match repo_data.as_ref() {
            None => (ContentBlobKind::Full, ""),
            Some(repo_data) => {
                let ancestors = repo_data.head_ancestors();
                let closest_full_ancestor_position = ancestors.iter().position(|v| v.content_blob_kind == ContentBlobKind::Full);
                match closest_full_ancestor_position {
                    None => (ContentBlobKind::Full, ""),
                    Some(pos) if pos >= MAX_CONSECUTIVE_PATCHES => (ContentBlobKind::Full, ""),
                    Some(pos) => {
                        let closest_full_ancestor = ancestors[pos];
                        let blob_kind = ContentBlobKind::Patch(closest_full_ancestor.id);
                        (blob_kind, closest_full_ancestor.content_blob_file_name.as_str())
                    }
                }
            }
        }
    };

    let new_version_id = VersionId::new();

    let content_blob_file_name = new_version_id.to_file_name() + "_content";
    let content_blob_file_path = repo_paths.repository_dir.join(&content_blob_file_name);

    match content_blob_kind {
        ContentBlobKind::Full => {
            fs::copy(&repo_paths.versioned_file, content_blob_file_path)?;
        }
        ContentBlobKind::Patch(_) => {
            let base_blob_file_path = repo_paths.repository_dir.join(&base_content_blob_file_name);
            xdelta3::create_patch(base_blob_file_path.as_path(), &repo_paths.versioned_file, content_blob_file_path.as_path())?;
        }
    }

    let versioned_file_is_image = repo_paths
        .versioned_file
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| known_file_types::is_image(extension))
        .unwrap_or(false);

    let preview_blob_file_name = if versioned_file_is_image {
        let preview_blob_file_name = new_version_id.to_file_name() + "_preview";
        let preview_blob_file_path = repo_paths.repository_dir.join(&preview_blob_file_name);
        image_magick::create_preview(repo_paths.versioned_file.as_path(), preview_blob_file_path.as_path())?;
        Some(preview_blob_file_name)
    } else {
        None
    };

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(xxh3_128),
        versioned_file_xxh3_128: xxh3_128,
        description: description.to_string(),
        parent: repo_data.as_ref().map(|data| data.head_version().id),
        content_blob_file_name,
        content_blob_kind,
        preview_blob_file_name,
    };

    let new_head = Head::Branch(branch.to_string());

    let mut owned_repo_data;
    let repo_data = match repo_data {
        Some(repo_data) => {
            repo_data.head = new_head;
            repo_data.versions.push(new_version);
            repo_data.branches.insert(branch, new_version_id);
            repo_data
        }
        None => {
            owned_repo_data = RepositoryData {
                head: new_head,
                versions: vec![new_version],
                branches: HashMap::from([(branch, new_version_id)]),
            };
            &mut owned_repo_data
        }
    };

    write_data_file(&repo_data, repo_paths)?;

    Ok(CommitResult::Ok)
}

pub fn has_uncommitted_changes(repo_paths: &RepositoryPaths, repo_data: &RepositoryData) -> io::Result<bool> {
    let versioned_file = File::open(&repo_paths.versioned_file)?;

    let current_xxh3_128 = hash::xxh3_128(&versioned_file)?;

    Ok(repo_data.head_version().versioned_file_xxh3_128 != current_xxh3_128)
}

pub fn discard(repo_paths: &RepositoryPaths, repo_data: &RepositoryData) -> io::Result<()> {
    let head_version = repo_data.head_version();
    set_versioned_file_to_version(repo_paths, repo_data, &head_version)?;
    Ok(())
}

pub enum CheckOutResult {
    Ok,
    BlockedByUncommittedChanges,
    InvalidTarget,
}

pub fn check_out(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, target: &str) -> io::Result<CheckOutResult> {
    let has_uncommitted_changes = has_uncommitted_changes(repo_paths, repo_data)?;

    if has_uncommitted_changes {
        return Ok(CheckOutResult::BlockedByUncommittedChanges);
    }

    let new_head = match resolve_target(repo_data, target) {
        TargetResult::Invalid => return Ok(CheckOutResult::InvalidTarget),
        TargetResult::Branch(branch) => Head::Branch(branch.to_string()),
        TargetResult::Version(version) => Head::Version(version.id),
    };

    repo_data.head = new_head;
    let new_head_version = repo_data.head_version();

    write_data_file(repo_data, repo_paths)?;
    set_versioned_file_to_version(repo_paths, repo_data, new_head_version)?;

    Ok(CheckOutResult::Ok)
}

fn write_data_file(data: &RepositoryData, paths: &RepositoryPaths) -> io::Result<()> {
    if !data.valid() {
        panic!("Repository data is not valid: {:#?}", data);
    }

    let data_file_content = serde_json::to_string_pretty(data)?;
    fs::write(&paths.data_file, data_file_content)
}

fn set_versioned_file_to_version(paths: &RepositoryPaths, data: &RepositoryData, version: &Version) -> io::Result<()> {
    let blob_path = paths.repository_dir.join(&version.content_blob_file_name);

    match version.content_blob_kind {
        ContentBlobKind::Full => {
            fs::copy(&blob_path, &paths.versioned_file)?;
        }
        ContentBlobKind::Patch(base_version_id) => {
            let base_version = data.version(base_version_id).expect("Version referenced by patch must exist");
            let base_version_blob_path = paths.repository_dir.join(&base_version.content_blob_file_name);
            xdelta3::apply_patch(base_version_blob_path.as_path(), blob_path.as_path(), &paths.versioned_file)?;
        }
    }

    Ok(())
}

enum TargetResult<'b, 'v> {
    Branch(&'b str),
    Version(&'v Version),
    Invalid,
}

fn resolve_target<'b, 'v>(repo_data: &'v RepositoryData, target: &'b str) -> TargetResult<'b, 'v> {
    if repo_data.branches.contains_key(target) {
        return TargetResult::Branch(target);
    }

    let target_as_version_id = VersionId::from_bs58(target);

    if let Some(target_as_version_id) = target_as_version_id {
        let version = repo_data.versions.iter().find(|v| v.id == target_as_version_id);
        if let Some(version) = version {
            return TargetResult::Version(version);
        }
    }

    let mut versions: Vec<_> = repo_data.versions.iter().collect();
    versions.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));

    let version = versions.iter().find(|v| nickname_matches(&v.nickname, target));

    if let Some(version) = version {
        return TargetResult::Version(version);
    }

    TargetResult::Invalid
}

fn nickname_matches(nickname: &str, input: &str) -> bool {
    if nickname.eq_ignore_ascii_case(input) {
        return true;
    }

    let nickname_chars_without_dash = nickname.chars().filter(|c| c != &'-');

    if nickname_chars_without_dash.eq(input.chars()) {
        return true;
    }

    fn nickname_matches_initials(nickname: &str, input: &str) -> bool {
        if input.len() != 2 {
            return false;
        }

        let input_initials_first = input.chars().nth(0).unwrap();
        let input_initials_second = input.chars().nth(1).unwrap();

        let index_of_dash = nickname.find('-').unwrap();
        let nickname_initials_first = nickname.chars().nth(0).unwrap();
        let nickname_initials_second = nickname.chars().nth(index_of_dash + 1).unwrap();

        input_initials_first == nickname_initials_first && input_initials_second == nickname_initials_second
    }

    nickname_matches_initials(nickname, input)
}
