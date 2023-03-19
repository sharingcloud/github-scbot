use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalAccountRight {
    pub username: String,
    pub repository_id: u64,
}
