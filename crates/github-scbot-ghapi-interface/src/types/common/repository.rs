use serde::{Deserialize, Serialize};

use super::GhUser;

/// GitHub Repository.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct GhRepository {
    /// Name.
    pub name: String,
    /// Full name.
    pub full_name: String,
    /// Owner.
    pub owner: GhUser,
}
