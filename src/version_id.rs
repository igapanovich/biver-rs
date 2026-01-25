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
    pub fn bs58(&self) -> String {
        bs58::encode(self.0.as_bytes()).into_string()
    }
    pub fn from_bs58(bs58: &str) -> Option<Self> {
        bs58::decode(bs58).into_vec().ok().and_then(|bytes| Uuid::from_slice(&bytes).ok()).map(Self)
    }
}
