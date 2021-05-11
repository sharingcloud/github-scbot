//! Pull request API module.

use github_scbot_conf::Config;
use github_scbot_types::pulls::{GhMergeStrategy, GhPullRequest};

use crate::{
    utils::{get_client, is_client_enabled},
    ApiError, Result,
};

/// Get pull request from ID.
pub async fn get_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<GhPullRequest> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;

        let data: GhPullRequest = client
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
        Err(ApiError::MissingPullRequest(
            format!("{}/{}", repository_owner, repository_name),
            pr_number,
        ))
    }
}

/// Get pull request last commit SHA.
pub async fn get_pull_request_sha(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<String> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
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
pub async fn merge_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    commit_title: &str,
    commit_message: &str,
    merge_strategy: GhMergeStrategy,
) -> Result<()> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
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
            403 => Err(ApiError::MergeError("Forbidden".to_string())),
            404 => Err(ApiError::MergeError("Not found".to_string())),
            405 => Err(ApiError::MergeError("Not mergeable".to_string())),
            409 => Err(ApiError::MergeError("Conflicts".to_string())),
            _ => Ok(()),
        };
    }

    Ok(())
}
