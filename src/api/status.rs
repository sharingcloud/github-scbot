//! Status API module.

use super::{errors::Result, get_client};
use crate::types::status::StatusState;

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
    repository_owner: &str,
    repository_name: &str,
    commit_sha: &str,
    status: StatusState,
    title: &str,
    body: &str,
) -> Result<()> {
    if !cfg!(test) {
        let client = get_client().await?;
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
