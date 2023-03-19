use serde::{Deserialize, Serialize};

use super::GhUser;

/// GitHub Application.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default, Clone)]
pub struct GhApplication {
    /// Slug name.
    pub slug: String,
    /// Owner.
    pub owner: GhUser,
    /// Name.
    pub name: String,
}
