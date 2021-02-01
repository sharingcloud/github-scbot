//! Comments API module.

use super::{errors::Result, get_client};

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
    if cfg!(test) {
        Ok(0)
    } else {
        let client = get_client().await?;
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);

        let comment = client
            .issues(repository_owner, repository_name)
            .create_comment(pr_number, final_body)
            .await?;

        Ok(comment.id)
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
    if !cfg!(test) {
        let client = get_client().await?;
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);

        client
            .issues(repository_owner, repository_name)
            .update_comment(comment_id, final_body)
            .await?;
    }

    Ok(comment_id)
}
