//! Comments API module.

use github_scbot_types::issues::GHReactionType;
use tracing::error;

use super::{errors::Result, get_client, is_client_enabled};
use crate::get_client_builder;

const BOT_COMMENT_SIGNATURE: &str = "_Beep boop, i'm a bot!_ :robot:";

/// Post a comment to a pull request.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `body` - Comment body
pub async fn post_comment(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    body: &str,
) -> Result<u64> {
    if is_client_enabled() {
        let client = get_client()?;
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);

        let comment = client
            .issues(repository_owner, repository_name)
            .create_comment(pr_number, final_body)
            .await?;

        Ok(comment.id)
    } else {
        Ok(0)
    }
}

/// Update a pull request comment.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `comment_id` - Comment ID
pub async fn update_comment(
    repository_owner: &str,
    repository_name: &str,
    comment_id: u64,
    body: &str,
) -> Result<u64> {
    if is_client_enabled() {
        let client = get_client()?;
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);

        client
            .issues(repository_owner, repository_name)
            .update_comment(comment_id, final_body)
            .await?;
    }

    Ok(comment_id)
}

/// Add reaction emoji to comment.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `comment_id` - Comment ID
/// * `reaction_type` - Reaction type
pub async fn add_reaction_to_comment(
    repository_owner: &str,
    repository_name: &str,
    comment_id: u64,
    reaction_type: GHReactionType,
) -> Result<()> {
    if is_client_enabled() {
        let client = get_client_builder().add_preview("squirrel-girl").build()?;
        let body = serde_json::json!({
            "content": reaction_type.to_str()
        });

        let data = client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/issues/comments/{}/reactions",
                    repository_owner, repository_name, comment_id
                ))?,
                Some(&body),
            )
            .await?;

        if data.status() != 201 {
            error!(
                "Could not add reaction to comment: status code {}",
                data.status()
            );
        }
    }

    Ok(())
}
