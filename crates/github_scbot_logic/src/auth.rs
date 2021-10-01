//! Authentication logic module.

use github_scbot_database::models::{IDatabaseAdapter, PullRequestModel};

use crate::Result;

/// Auth logic.
pub struct AuthLogic;

impl AuthLogic {
    /// List known admin usernames.
    pub async fn list_known_admin_usernames(
        db_adapter: &dyn IDatabaseAdapter,
    ) -> Result<Vec<String>> {
        Ok(db_adapter
            .account()
            .list_admin_accounts()
            .await?
            .iter()
            .map(|acc| acc.username.clone())
            .collect())
    }

    /// Check if user has right on pull request.
    pub fn has_right_on_pull_request(
        username: &str,
        pr_model: &PullRequestModel,
        known_admins: &[String],
    ) -> bool {
        Self::is_admin(username, known_admins) || pr_model.creator() == username
    }

    /// Check if user is admin.
    pub fn is_admin(username: &str, known_admins: &[String]) -> bool {
        known_admins.contains(&username.into())
    }
}
