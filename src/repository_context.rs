use crate::repository_data::RepositoryData;
use crate::repository_paths::RepositoryPaths;

pub struct RepositoryContext {
    pub paths: RepositoryPaths,
    pub data: RepositoryData,
}
