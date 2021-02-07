//! Pull request API module.

use github_scbot_types::pulls::{GHMergeStrategy, GHPullRequest};

use crate::{
    utils::{get_client, is_client_enabled},
    APIError, Result,
};

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
) -> Result<GHPullRequest> {
    if is_client_enabled() {
        let client = get_client().await?;

        let data: GHPullRequest = client
            .get(
                format!(
                    "/repos/{owner}/{name}/pulls/{pr_number}",
                    owner = repository_owner,
                    name = repository_name,
                    pr_number = pr_number
                ),
                None::<&()>,
            )
            .await?;
        Ok(data)
    } else {
        Err(APIError::MissingPullRequest(
            format!("{}/{}", repository_owner, repository_name),
            pr_number,
        ))
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
    if is_client_enabled() {
        let client = get_client().await?;
        let data = client
            .pulls(repository_owner, repository_name)
            .get(pr_number)
            .await?;
        Ok(data.head.sha)
    } else {
        Ok("abcdef".to_string())
    }
}

/// Merge pull request.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - PR number
/// * `commit_title` - Commit title
/// * `commit_message` - Commit message
/// * `commit_message` - Commit message
pub async fn merge_pull_request(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    commit_title: &str,
    commit_message: &str,
    merge_strategy: GHMergeStrategy,
) -> Result<()> {
    if is_client_enabled() {
        let client = get_client().await?;
        let body = serde_json::json!({
            "commit_title": commit_title,
            "commit_message": commit_message,
            "merge_method": merge_strategy.to_string()
        });

        let response = client
            ._put(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/merge",
                    repository_owner, repository_name, pr_number
                ))?,
                Some(&body),
            )
            .await?;

        let code: u16 = response.status().into();
        return match code {
            403 => Err(APIError::MergeError("Forbidden".to_string())),
            404 => Err(APIError::MergeError("Not found".to_string())),
            405 => Err(APIError::MergeError("Not mergeable".to_string())),
            409 => Err(APIError::MergeError("Conflicts".to_string())),
            _ => Ok(()),
        };
    }

    Ok(())
}
