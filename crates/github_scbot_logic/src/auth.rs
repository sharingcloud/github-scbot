//! Authentication logic module.

use github_scbot_database2::DbService;
use github_scbot_types::common::GhUserPermission;

use crate::Result;

/// Auth logic.
pub struct AuthLogic;

impl AuthLogic {
    /// List known admin usernames.
    pub async fn list_known_admin_usernames(
        db_adapter: &dyn DbService,
    ) -> Result<Vec<String>> {
        Ok(db_adapter
            .accounts()
            .list_admins()
            .await?
            .iter()
            .map(|acc| acc.username().into())
            .collect())
    }

    /// Check if user has write right.
    pub fn has_write_right(
        username: &str,
        user_permission: GhUserPermission,
        known_admins: &[String],
    ) -> bool {
        Self::is_admin(username, known_admins) || user_permission.can_write()
    }

    /// Check if user is admin.
    pub fn is_admin(username: &str, known_admins: &[String]) -> bool {
        known_admins.contains(&username.into())
    }
}
