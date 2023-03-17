use serde::{Deserialize, Serialize};

/// GitHub Label.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct GhLabel {
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
    /// Description.
    pub description: Option<String>,
}
