use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryData {
    pub head: Option<VersionId>,
    pub versions: Vec<Version>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub creation_time: DateTime<Utc>,
    pub description: String,
    pub parent: Option<VersionId>,
    pub blob_file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionId(Uuid);
impl VersionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn to_file_name(&self) -> String {
        self.0.to_string()
    }
}
impl Clone for VersionId {
    fn clone(&self) -> Self {
        VersionId(self.0)
    }
}
