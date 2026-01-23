use crate::repository_context::RepositoryContext;
use crate::repository_data::{RepositoryData, Version, VersionId};
use crate::repository_paths::RepositoryPaths;
use chrono::Utc;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fs, io};

pub fn init_and_get_repository_context(versioned_file_path: &PathBuf) -> Result<RepositoryContext, io::Error> {
    let versioned_file_path = fs::canonicalize(versioned_file_path)?;
    let paths = repository_paths(versioned_file_path);
    let data = init_and_read_repository_data(&paths)?;
    Ok(RepositoryContext { paths, data })
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

fn init_and_read_repository_data(repository_paths: &RepositoryPaths) -> Result<RepositoryData, io::Error> {
    if !repository_paths.repository_dir.exists() {
        fs::create_dir(&repository_paths.repository_dir)?;
    }

    let repository_data = if !repository_paths.data_file.exists() {
        let repository_data = initial_repository_data();
        let data_file_contents = serde_json::to_string_pretty(&repository_data)?;
        fs::write(&repository_paths.data_file, data_file_contents)?;
        repository_data
    } else {
        let data_file_contents = fs::read(&repository_paths.data_file)?;
        serde_json::from_slice(&data_file_contents)?
    };

    Ok(repository_data)
}

fn initial_repository_data() -> RepositoryData {
    RepositoryData { head: None, versions: vec![] }
}

pub fn commit_version(mut repo: RepositoryContext, description: String) -> Result<(), io::Error> {
    if repo.data.head.is_none() && !repo.data.versions.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Only the initial version can have no parent."));
    }

    let new_version_id = VersionId::new();

    let blob_file_name = new_version_id.to_file_name();
    let blob_file_path = repo.paths.repository_dir.join(blob_file_name.clone());

    fs::copy(repo.paths.versioned_file, blob_file_path)?;

    let new_version = Version {
        id: new_version_id,
        parent: repo.data.head.clone(),
        description,
        creation_time: Utc::now(),
        blob_file_name,
    };

    repo.data.head = Some(new_version.id.clone());
    repo.data.versions.push(new_version);

    let new_data_file_content = serde_json::to_string_pretty(&repo.data)?;
    fs::write(&repo.paths.data_file, new_data_file_content)?;

    Ok(())
}
