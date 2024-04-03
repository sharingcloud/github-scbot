use serde::{Deserialize, Serialize};

/// GitHub User.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct GhUser {
    /// Username.
    pub login: String,
}
