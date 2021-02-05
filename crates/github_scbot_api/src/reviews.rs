//! Reviews API module.

use tracing::error;

use crate::{
    utils::{get_client, is_client_enabled},
    Result,
};

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
    if is_client_enabled() {
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
    if is_client_enabled() {
        let body = serde_json::json!({ "reviewers": reviewers });

        let client = get_client().await?;
        let url = client.absolute_url(format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            repository_owner, repository_name, pr_number
        ))?;
        let builder = client
            .request_builder(&url.into_string(), http::Method::DELETE)
            .json(&body)
            .header(http::header::ACCEPT, octocrab::format_media_type("json"));

        let response = client.execute(builder).await?;
        if response.status() != 200 {
            error!(
                "Could not remove reviewers {:?} for pull request {}/{}: status code: {}",
                reviewers,
                repository_owner,
                repository_name,
                response.status()
            );
        }
    }

    Ok(())
}
