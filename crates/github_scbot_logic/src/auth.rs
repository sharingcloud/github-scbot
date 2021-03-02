//! Authentication logic module.

use github_scbot_database::{
    models::{AccountModel, PullRequestModel},
    DbConn,
};

use crate::Result;

/// List known admin usernames.
///
/// # Arguments
///
/// * `conn` - Database connection
pub fn list_known_admin_usernames(conn: &DbConn) -> Result<Vec<String>> {
    Ok(AccountModel::list_admin_accounts(conn)?
        .iter()
        .map(|acc| acc.username.clone())
        .collect())
}

/// Check if user has right on pull request.
///
/// # Arguments
///
/// * `username` - Target username
/// * `pr_model` - Pull request model
/// * `known_admins` - Known admin usernames
pub fn has_right_on_pull_request(
    username: &str,
    pr_model: &PullRequestModel,
    known_admins: &[String],
) -> bool {
    known_admins.contains(&username.into()) || pr_model.creator == username
}
