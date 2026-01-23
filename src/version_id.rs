use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl PartialEq for VersionId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Clone for VersionId {
    fn clone(&self) -> Self {
        VersionId(self.0)
    }
}
