use std::ffi::OsString;
use std::path::PathBuf;

pub struct RepositoryPaths {
    pub versioned_file: PathBuf,
    pub repository_dir: PathBuf,
    pub data_file: PathBuf,
}

impl RepositoryPaths {
    pub fn from_versioned_file_path(versioned_file_path: PathBuf) -> Self {
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

    pub fn file_path(&self, file_name: &str) -> PathBuf {
        self.repository_dir.join(&file_name)
    }
}
