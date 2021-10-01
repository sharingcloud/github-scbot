//! Comments API module.

use github_scbot_types::issues::GhReactionType;

use crate::{adapter::IAPIAdapter, Result};

const BOT_COMMENT_SIGNATURE: &str = "_Beep boop, i'm a bot!_ :robot:";

/// Comment API.
pub struct CommentApi;

impl CommentApi {
    /// Post a comment to a pull request.
    pub async fn post_comment(
        adapter: &dyn IAPIAdapter,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<u64> {
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);
        adapter
            .comments_post(repository_owner, repository_name, pr_number, &final_body)
            .await
    }

    /// Update a pull request comment.
    pub async fn update_comment(
        adapter: &dyn IAPIAdapter,
        repository_owner: &str,
        repository_name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        let final_body = format!("{}\n\n{}", body, BOT_COMMENT_SIGNATURE);
        adapter
            .comments_update(repository_owner, repository_name, comment_id, &final_body)
            .await
    }

    /// Add reaction emoji to comment.
    pub async fn add_reaction_to_comment(
        adapter: &dyn IAPIAdapter,
        repository_owner: &str,
        repository_name: &str,
        comment_id: u64,
        reaction_type: GhReactionType,
    ) -> Result<()> {
        if comment_id > 0 {
            adapter
                .comment_reactions_add(repository_owner, repository_name, comment_id, reaction_type)
                .await?;
        }

        Ok(())
    }
}
