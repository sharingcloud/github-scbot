use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredReviewer {
    pub pull_request_id: u64,
    pub username: String,
}
