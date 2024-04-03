use serde::{Deserialize, Serialize};

/// GitHub User permission.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GhUserPermission {
    /// Admin.
    Admin,
    /// Write.
    Write,
    /// Read.
    Read,
    /// None.
    None,
}

impl GhUserPermission {
    /// Can write?
    pub fn can_write(self) -> bool {
        matches!(self, Self::Admin | Self::Write)
    }
}
