//! Comments API module.

use crate::{types::GhReactionType, ApiService, Result};

const BOT_COMMENT_SIGNATURE: &str = "_Beep boop, i'm a bot!_ :robot:";

/// Comment API.
pub struct CommentApi;

impl CommentApi {
    fn build_bot_comment(source: &str) -> String {
        format!(
            "{}\n\n{} _(github-scbot {})_",
            source,
            BOT_COMMENT_SIGNATURE,
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Post a comment to a pull request.
    pub async fn post_comment(
        adapter: &dyn ApiService,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<u64> {
        let final_body = Self::build_bot_comment(body);
        adapter
            .comments_post(repository_owner, repository_name, pr_number, &final_body)
            .await
    }

    /// Update a pull request comment.
    pub async fn update_comment(
        adapter: &dyn ApiService,
        repository_owner: &str,
        repository_name: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<u64> {
        let final_body = Self::build_bot_comment(body);
        adapter
            .comments_update(repository_owner, repository_name, comment_id, &final_body)
            .await
    }

    /// Add reaction emoji to comment.
    pub async fn add_reaction_to_comment(
        adapter: &dyn ApiService,
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
