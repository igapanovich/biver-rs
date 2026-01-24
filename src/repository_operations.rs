use crate::repository_context::RepositoryContext;
use crate::repository_data::{RepositoryData, Version};
use crate::repository_paths::RepositoryPaths;
use crate::version_id::VersionId;
use crate::{hash, nickname};
use chrono::Utc;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

pub enum RepositoryContextResult {
    Initialized(RepositoryContext),
    NotInitialized(RepositoryPaths),
}

pub fn repository_context(versioned_file_path: &PathBuf) -> Result<RepositoryContextResult, io::Error> {
    let versioned_file_path = fs::canonicalize(versioned_file_path)?;
    let paths = repository_paths(versioned_file_path);
    let data = repository_data(&paths)?;

    match data {
        Some(data) => Ok(RepositoryContextResult::Initialized(RepositoryContext { paths, data })),
        None => Ok(RepositoryContextResult::NotInitialized(paths)),
    }
}

fn repository_paths(versioned_file_path: PathBuf) -> RepositoryPaths {
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

fn repository_data(repository_paths: &RepositoryPaths) -> Result<Option<RepositoryData>, io::Error> {
    if !repository_paths.data_file.exists() {
        return Ok(None);
    }

    let data_file_contents = fs::read(&repository_paths.data_file)?;
    let repository_data = serde_json::from_slice(&data_file_contents)?;

    Ok(repository_data)
}

pub enum CommitResult {
    Ok,
    NothingToCommit,
}

pub fn commit_initial_version(paths: &RepositoryPaths, description: &str) -> Result<CommitResult, io::Error> {
    if !fs::exists(&paths.repository_dir)? {
        fs::create_dir(&paths.repository_dir)?;
    } else if fs::exists(&paths.data_file)? {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "The data file already exists."));
    }

    commit_version_common(paths, None, description)
}

pub fn commit_version(repo: &RepositoryContext, description: &str) -> Result<CommitResult, io::Error> {
    commit_version_common(&repo.paths, Some(&repo.data), description)
}

fn commit_version_common(paths: &RepositoryPaths, data: Option<&RepositoryData>, description: &str) -> Result<CommitResult, io::Error> {
    let versioned_file = File::open(&paths.versioned_file)?;

    let xxh3_128 = hash::xxh3_128(&versioned_file)?;

    if let Some(data) = &data
        && data.head_version().versioned_file_xxh3_128 == xxh3_128
    {
        return Ok(CommitResult::NothingToCommit);
    }

    let new_version_id = VersionId::new();

    let blob_file_name = new_version_id.to_file_name();
    let blob_file_path = paths.repository_dir.join(&blob_file_name);

    fs::copy(&paths.versioned_file, blob_file_path)?;

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(xxh3_128),
        versioned_file_xxh3_128: xxh3_128,
        description: description.to_string(),
        parent: data.as_ref().map(|data| data.head),
        blob_file_name,
    };

    let data = match data {
        Some(data) => {
            let mut data = data.clone();
            data.head = new_version.id;
            data.versions.push(new_version);
            data
        }
        None => RepositoryData {
            head: new_version.id,
            versions: vec![new_version],
        },
    };

    let new_data_file_content = serde_json::to_string_pretty(&data)?;
    fs::write(&paths.data_file, new_data_file_content)?;

    Ok(CommitResult::Ok)
}
