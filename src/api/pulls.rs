//! Pull request API module.

use super::{errors::Result, get_client};

/// Get pull request from ID.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
pub async fn get_pull_request(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<octocrab::models::pulls::PullRequest> {
    if cfg!(test) {
        panic!("get_pull_request disabled in test mode")
    } else {
        let client = get_client().await?;

        client
            .pulls(repository_owner, repository_name)
            .get(pr_number)
            .await
            .map_err(Into::into)
    }
}

/// Get pull request last commit SHA.
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
pub async fn get_pull_request_sha(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<String> {
    if cfg!(test) {
        Ok("abcdef".to_string())
    } else {
        tracing::info!("Will get_client");
        let client = get_client().await?;

        tracing::info!("Will get PR data");
        let data = client
            .pulls(repository_owner, repository_name)
            .get(pr_number)
            .await?;
        Ok(data.head.sha)
    }
}
