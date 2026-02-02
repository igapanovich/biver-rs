use crate::env::Env;
use crate::repository_data::{ContentBlob, RepositoryData};
use crate::repository_paths::RepositoryPaths;
use crate::{image_magick, xdelta3};
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::{fs, io};

pub enum RepositoryDataResult {
    Initialized(RepositoryData),
    NotInitialized,
}

pub fn read_data(repository_paths: &RepositoryPaths) -> io::Result<RepositoryDataResult> {
    if !repository_paths.data_file.exists() {
        return Ok(RepositoryDataResult::NotInitialized);
    }

    let data_file_contents = fs::read(&repository_paths.data_file)?;
    let repository_data = serde_json::from_slice(&data_file_contents)?;

    Ok(RepositoryDataResult::Initialized(repository_data))
}

pub fn write_data(paths: &RepositoryPaths, data: &RepositoryData) -> io::Result<()> {
    if !data.valid() {
        panic!("Repository data is not valid: {:#?}", data);
    }

    let backup1 = paths.file_path("data_backup1.json");
    let backup2 = paths.file_path("data_backup2.json");
    let backup3 = paths.file_path("data_backup3.json");
    let backup4 = paths.file_path("data_backup4.json");
    let backup5 = paths.file_path("data_backup5.json");

    rotate_backup(&backup4, &backup5, Duration::from_hours(24))?;
    rotate_backup(&backup3, &backup4, Duration::from_hours(5))?;
    rotate_backup(&backup2, &backup3, Duration::from_hours(1))?;
    rotate_backup(&backup1, &backup2, Duration::from_mins(5))?;
    rotate_backup(&paths.data_file, &backup1, Duration::from_secs(10))?;

    let data_file_content = serde_json::to_string_pretty(data)?;
    fs::write(&paths.data_file, data_file_content)?;

    Ok(())
}

pub fn store_version_content(env: &Env, repo_paths: &RepositoryPaths, content_blob: &ContentBlob, content_to_store_path: &Path) -> io::Result<()> {
    match content_blob {
        ContentBlob::Full { full_blob_file_name } => {
            let full_blob_file_path = repo_paths.file_path(&full_blob_file_name);
            fs::copy(content_to_store_path, full_blob_file_path)?;
        }

        ContentBlob::Patch {
            base_blob_file_name,
            patch_blob_file_name,
        } => {
            let patch_blob_file_path = repo_paths.file_path(&patch_blob_file_name);
            let base_blob_file_path = repo_paths.file_path(&base_blob_file_name);
            xdelta3::create_patch(env, &base_blob_file_path, content_to_store_path, &patch_blob_file_path)?;
        }
    }

    Ok(())
}

pub fn extract_version_content(env: &Env, repo_paths: &RepositoryPaths, content_blob: &ContentBlob, destination_path: &Path) -> io::Result<()> {
    match content_blob {
        ContentBlob::Full { full_blob_file_name } => {
            let full_blob_file_path = repo_paths.file_path(&full_blob_file_name);
            fs::copy(&full_blob_file_path, destination_path)?;
        }

        ContentBlob::Patch {
            base_blob_file_name,
            patch_blob_file_name,
        } => {
            let patch_blob_file_path = repo_paths.file_path(&patch_blob_file_name);
            let base_blob_file_path = repo_paths.file_path(&base_blob_file_name);
            xdelta3::apply_patch(env, &base_blob_file_path, &patch_blob_file_path, destination_path)?;
        }
    }

    Ok(())
}

pub fn store_version_preview(env: &Env, repo_paths: &RepositoryPaths, preview_blob_file_name: &str, content_to_store_path: &Path) -> io::Result<()> {
    let preview_blob_file_path = repo_paths.file_path(preview_blob_file_name);

    image_magick::create_preview(env, content_to_store_path, preview_blob_file_path.as_path())?;

    Ok(())
}

fn rotate_backup(previous: &Path, next: &Path, interval: Duration) -> io::Result<()> {
    if !previous.exists() {
        return Ok(());
    }

    if next.exists() && next.metadata()?.modified()? > SystemTime::now() - interval {
        return Ok(());
    }

    fs::copy(previous, next)?;

    Ok(())
}
