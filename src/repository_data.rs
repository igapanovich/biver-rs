use crate::version_id::VersionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryData {
    pub head: Head,
    pub branches: HashMap<String, VersionId>,
    pub versions: Vec<Version>,
}

impl RepositoryData {
    pub fn version(&self, id: &VersionId) -> Option<&Version> {
        self.versions.iter().find(|v| &v.id == id)
    }

    pub fn head_version(&self) -> &Version {
        let head_version = match &self.head {
            Head::Branch(branch) => {
                let head_branch = self.branches.get(branch).expect("The head branch must always exist.");
                self.version(head_branch)
            }
            Head::Version(version_id) => self.version(version_id),
        };

        head_version.expect("The head version must always exist.")
    }

    pub fn branch_on_version(&self, version_id: &VersionId) -> Option<&str> {
        self.branches
            .iter()
            .find_map(|(branch, version)| if version == version_id { Some(branch.deref()) } else { None })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub creation_time: DateTime<Utc>,
    pub nickname: String,
    pub versioned_file_xxh3_128: u128,
    pub description: String,
    pub parent: Option<VersionId>,
    pub blob_file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Head {
    Branch(String),
    Version(VersionId),
}
