use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionId(Uuid);

impl VersionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn to_file_name(&self) -> String {
        self.0.to_string()
    }
}
