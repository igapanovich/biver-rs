use crate::repository_data::{Head, RepositoryData, Version};
use crate::repository_operations::RepositoryDataResult::{Initialized, NotInitialized};
use crate::repository_paths::RepositoryPaths;
use crate::version_id::VersionId;
use crate::{hash, nickname};
use chrono::Utc;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

const DEFAULT_BRANCH: &str = "main";

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
        return Ok(NotInitialized);
    }

    let data_file_contents = fs::read(&repository_paths.data_file)?;
    let repository_data = serde_json::from_slice(&data_file_contents)?;

    Ok(Initialized(repository_data))
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

    let new_version_id = VersionId::new();

    let blob_file_name = new_version_id.to_file_name();
    let blob_file_path = repo_paths.repository_dir.join(&blob_file_name);

    fs::copy(&repo_paths.versioned_file, blob_file_path)?;

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(xxh3_128),
        versioned_file_xxh3_128: xxh3_128,
        description: description.to_string(),
        parent: repo_data.as_ref().map(|data| data.head_version().id),
        blob_file_name,
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
    let head_blob_file_path = repo_paths.repository_dir.join(&head_version.blob_file_name);
    fs::copy(&head_blob_file_path, &repo_paths.versioned_file)?;
    Ok(())
}

fn write_data_file(data: &RepositoryData, paths: &RepositoryPaths) -> io::Result<()> {
    if !data.valid() {
        panic!("Repository data is not valid: {:#?}", data);
    }

    let data_file_content = serde_json::to_string_pretty(data)?;
    fs::write(&paths.data_file, data_file_content)
}
