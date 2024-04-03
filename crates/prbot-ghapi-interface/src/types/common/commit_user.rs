use serde::Deserialize;

/// GitHub Commit user.
#[derive(Debug, Deserialize)]
pub struct GhCommitUser {
    /// Name.
    pub name: String,
    /// Email.
    pub email: String,
    /// Username.
    pub username: Option<String>,
}
