//! Pull requests API module

use super::errors::Result;
use super::get_client;

pub async fn get_pull_request(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
) -> Result<octocrab::models::pulls::PullRequest> {
    let client = get_client().await?;

    client
        .pulls(repo_owner, repo_name)
        .get(pr_number)
        .await
        .map_err(Into::into)
}

pub async fn get_pull_request_sha(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
) -> Result<String> {
    tracing::info!("Will get_client");
    let client = get_client().await?;

    tracing::info!("Will get PR data");
    let data = client.pulls(repo_owner, repo_name).get(pr_number).await?;

    Ok(data.head.sha)
}
