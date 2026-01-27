use std::path::PathBuf;

pub struct RepositoryPaths {
    pub versioned_file: PathBuf,
    pub repository_dir: PathBuf,
    pub data_file: PathBuf,
}

impl RepositoryPaths {
    pub fn file_path(&self, file_name: &str) -> PathBuf {
        self.repository_dir.join(&file_name)
    }
}
