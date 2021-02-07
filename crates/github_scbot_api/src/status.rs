//! Status API module.

use github_scbot_core::Config;
use github_scbot_types::status::StatusState;

use crate::{
    utils::{get_client, is_client_enabled},
    Result,
};

const MAX_STATUS_DESCRIPTION_LEN: usize = 139;

/// Update status for repository.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `commit_sha` - Commit SHA
/// * `status` - Status state
/// * `title` - Status title
/// * `body` - Status body
pub async fn update_status_for_repository(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    commit_sha: &str,
    status: StatusState,
    title: &str,
    body: &str,
) -> Result<()> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
        let body = serde_json::json!({
            "state": status.to_str(),
            "description": body.chars().take(MAX_STATUS_DESCRIPTION_LEN).collect::<String>(),
            "context": title
        });

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/statuses/{}",
                    repository_owner, repository_name, commit_sha
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}
