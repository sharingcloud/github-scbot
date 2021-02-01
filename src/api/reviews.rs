//! Reviews API module.

use super::{errors::Result, get_client};

/// Request reviewers for a pull request.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `reviewers` - Reviewers names
pub async fn request_reviewers_for_pull_request(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    reviewers: &[String],
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
        let body = serde_json::json!({ "reviewers": reviewers });

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    repository_owner, repository_name, pr_number
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}

/// Remove requested reviewers for a pull request.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `reviewers` - Reviewers names
pub async fn remove_reviewers_for_pull_request(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    reviewers: &[String],
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
        let body = serde_json::json!({ "reviewers": reviewers });

        client
            ._delete(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    repository_owner, repository_name, pr_number
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}
