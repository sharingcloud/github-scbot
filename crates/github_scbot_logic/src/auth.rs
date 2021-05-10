//! Authentication logic module.

use github_scbot_database::{
    models::{AccountModel, PullRequestModel},
    DbConn,
};

use crate::Result;

/// List known admin usernames.
pub fn list_known_admin_usernames(conn: &DbConn) -> Result<Vec<String>> {
    Ok(AccountModel::list_admin_accounts(conn)?
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
    is_admin(username, known_admins) || pr_model.creator == username
}

/// Check uif user is admin.
pub fn is_admin(username: &str, known_admins: &[String]) -> bool {
    known_admins.contains(&username.into())
}
