use crate::version_id::VersionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryData {
    pub head: Option<VersionId>,
    pub versions: Vec<Version>,
}
impl RepositoryData {
    pub fn version(&self, id: &VersionId) -> Option<&Version> {
        self.versions.iter().find(|v| &v.id == id)
    }

    pub fn head_version(&self) -> Option<&Version> {
        self.head.as_ref().and_then(|head| self.version(&head))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub creation_time: DateTime<Utc>,
    pub versioned_file_xxh3_128: u128,
    pub description: String,
    pub parent: Option<VersionId>,
    pub blob_file_name: String,
}
