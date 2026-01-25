use crate::version_id::VersionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryData {
    pub head: Head,
    pub branches: HashMap<String, VersionId>,
    pub versions: Vec<Version>,
}

impl RepositoryData {
    pub fn version(&self, id: VersionId) -> Option<&Version> {
        self.versions.iter().find(|v| v.id == id)
    }

    pub fn head_version(&self) -> &Version {
        let head_version = match &self.head {
            Head::Branch(branch) => {
                let head_version_id = self.branches.get(branch).expect("The head branch must always exist.");
                self.version(*head_version_id)
            }
            Head::Version(version_id) => self.version(*version_id),
        };

        head_version.expect("The head version must always exist.")
    }

    pub fn branch_on_version(&self, version_id: &VersionId) -> Option<&str> {
        self.branches
            .iter()
            .find_map(|(branch, version)| if version == version_id { Some(branch.deref()) } else { None })
    }

    pub fn valid(&self) -> bool {
        let there_is_exactly_one_root = self.versions.iter().filter(|v| v.parent.is_none()).count() == 1;

        let there_are_no_invalid_parent_references = self.versions.iter().all(|v| {
            if let Some(parent) = &v.parent {
                self.versions.iter().any(|v2| v2.id == *parent)
            } else {
                true
            }
        });

        let head_reference_is_valid = match &self.head {
            Head::Branch(branch) => self.branches.contains_key(branch),
            Head::Version(version_id) => self.versions.iter().any(|v| v.id == *version_id),
        };

        let all_branches_reference_valid_versions = self.branches.values().all(|branch_version_id| self.versions.iter().any(|v| v.id == *branch_version_id));

        let no_two_branches_reference_the_same_version = {
            let distinct_branch_values: HashSet<&VersionId> = self.branches.values().collect();
            self.branches.values().count() == distinct_branch_values.len()
        };

        there_is_exactly_one_root
            && there_are_no_invalid_parent_references
            && head_reference_is_valid
            && all_branches_reference_valid_versions
            && no_two_branches_reference_the_same_version
    }

    pub fn head_ancestors(&self) -> Vec<&Version> {
        let head_version = self.head_version();
        let mut versions_from_head_to_root = vec![head_version];
        let mut current_version = head_version;

        while let Some(parent) = current_version.parent {
            current_version = self.version(parent).expect("The parent version must always exist.");
            versions_from_head_to_root.push(current_version);
        }

        versions_from_head_to_root
    }

    pub fn branch_leaf(&self, branch: &str) -> Option<&Version> {
        self.branches.get(branch).and_then(|version_id| self.version(*version_id))
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
    pub content_blob_file_name: String,
    pub content_blob_kind: ContentBlobKind,
    pub preview_blob_file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Head {
    Branch(String),
    Version(VersionId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentBlobKind {
    Full,
    Patch(VersionId),
}
