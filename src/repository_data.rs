use crate::version_id::VersionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

    pub fn valid(&self) -> bool {
        let there_is_exactly_one_root = self.versions.iter().filter(|v| v.parent.is_none()).count() == 1;

        let all_parent_references_are_valid = self.versions.iter().all(|v| {
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

        let all_versions_belong_to_branches = {
            let mut versions_belonging_to_branches = HashSet::new();

            for branch_leaf_id in self.branches.values() {
                for v in self.iter_version_and_ancestors(*branch_leaf_id) {
                    if !versions_belonging_to_branches.insert(v.id) {
                        break;
                    }
                }
            }

            versions_belonging_to_branches.len() == self.versions.len()
        };

        there_is_exactly_one_root
            && all_parent_references_are_valid
            && head_reference_is_valid
            && all_branches_reference_valid_versions
            && no_two_branches_reference_the_same_version
            && all_versions_belong_to_branches
    }

    pub fn iter_version_and_ancestors(&'_ self, version_id: VersionId) -> impl Iterator<Item = &'_ Version> {
        let version = self.version(version_id);
        VersionAndAncestors {
            repository_data: self,
            current_version: version,
        }
    }

    pub fn iter_head_and_ancestors(&'_ self) -> impl Iterator<Item = &'_ Version> {
        self.iter_version_and_ancestors(self.head_version().id)
    }

    pub fn iter_children(&self, version_id: VersionId) -> impl Iterator<Item = &'_ Version> {
        self.versions.iter().filter(move |v| v.parent == Some(version_id))
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
    pub versioned_file_length: u64,
    pub versioned_file_xxh3_128: u128,
    pub description: String,
    pub parent: Option<VersionId>,
    pub content_blob: ContentBlob,
    pub preview_blob_file_name: Option<String>,
}

impl Version {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Head {
    Branch(String),
    Version(VersionId),
}

impl Head {
    pub fn branch(&self) -> Option<&str> {
        match self {
            Head::Branch(branch) => Some(branch),
            Head::Version(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentBlob {
    Full { full_blob_file_name: String },
    Patch { base_blob_file_name: String, patch_blob_file_name: String },
}

pub struct VersionAndAncestors<'a> {
    repository_data: &'a RepositoryData,
    current_version: Option<&'a Version>,
}

impl<'a> Iterator for VersionAndAncestors<'a> {
    type Item = &'a Version;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_version {
            None => None,
            Some(version) => {
                self.current_version = version.parent.and_then(|parent_id| self.repository_data.version(parent_id));
                Some(version)
            }
        }
    }
}
