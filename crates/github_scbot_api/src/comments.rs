//! Comments API module.

use github_scbot_conf::Config;
use github_scbot_types::issues::GhReactionType;
use tracing::error;

use crate::{
    utils::{get_client, get_client_builder, is_client_enabled},
    Result,
};

const BOT_COMMENT_SIGNATURE: &str = "_Beep boop, i'm a bot!_ :robot:";

/// Post a comment to a pull request.
pub async fn post_comment(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    body: &str,
) -> Result<u64> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
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
pub async fn update_comment(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    comment_id: u64,
    body: &str,
) -> Result<u64> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);

        client
            .issues(repository_owner, repository_name)
            .update_comment(comment_id, final_body)
            .await?;
    }

    Ok(comment_id)
}

/// Add reaction emoji to comment.
pub async fn add_reaction_to_comment(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    comment_id: u64,
    reaction_type: GhReactionType,
) -> Result<()> {
    if is_client_enabled(config) {
        let client = get_client_builder(config)
            .await?
            .add_preview("squirrel-girl")
            .build()?;
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
                status_code = %data.status(),
                message = "Could not add reaction to comment",
            );
        }
    }

    Ok(())
}
