use github_scbot_api::{
    adapter::IAPIAdapter,
    comments::{post_comment, update_comment},
};
use github_scbot_database::models::{IDatabaseAdapter, PullRequestModel, RepositoryModel};

use super::SummaryTextGenerator;
use crate::{status::PullRequestStatus, Result};

/// Summary comment sender.
#[derive(Default)]
pub struct SummaryCommentSender;

impl SummaryCommentSender {
    /// Creates new summary comment sender.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates comment.
    pub async fn create(
        &self,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &mut PullRequestModel,
    ) -> Result<u64> {
        let pull_request_status =
            PullRequestStatus::from_database(db_adapter, repo_model, pr_model).await?;
        let status_comment = Self::generate_comment(&pull_request_status)?;
        self.post_github_comment(
            api_adapter,
            db_adapter,
            repo_model,
            pr_model,
            &status_comment,
        )
        .await
    }

    /// Update comment.
    pub async fn update(
        &self,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &mut PullRequestModel,
    ) -> Result<u64> {
        let pull_request_status =
            PullRequestStatus::from_database(db_adapter, repo_model, pr_model).await?;
        let status_comment = Self::generate_comment(&pull_request_status)?;

        // Re-fetch comment ID
        let comment_id = db_adapter
            .pull_request()
            .fetch_status_comment_id(pr_model.id)
            .await? as u64;
        if comment_id > 0 {
            if let Ok(comment_id) = update_comment(
                api_adapter,
                &repo_model.owner,
                &repo_model.name,
                comment_id,
                &status_comment,
            )
            .await
            {
                Ok(comment_id)
            } else {
                // Comment ID is no more used on GitHub, recreate a new status
                self.post_github_comment(
                    api_adapter,
                    db_adapter,
                    repo_model,
                    pr_model,
                    &status_comment,
                )
                .await
            }
        } else {
            // Too early, do not update the status comment
            Ok(0)
        }
    }

    /// Delete comment.
    pub async fn delete(
        &self,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &mut PullRequestModel,
    ) -> Result<()> {
        // Re-fetch comment ID
        let comment_id = db_adapter
            .pull_request()
            .fetch_status_comment_id(pr_model.id)
            .await? as u64;

        if comment_id > 0 {
            api_adapter
                .comments_delete(&repo_model.owner, &repo_model.name, comment_id)
                .await?;
        }

        Ok(())
    }

    fn generate_comment(pull_request_status: &PullRequestStatus) -> Result<String> {
        SummaryTextGenerator::generate(pull_request_status)
    }

    async fn post_github_comment(
        &self,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &mut PullRequestModel,
        comment: &str,
    ) -> Result<u64> {
        let comment_id = post_comment(
            api_adapter,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            comment,
        )
        .await?;

        pr_model.set_status_comment_id(comment_id);
        db_adapter.pull_request().save(pr_model).await?;
        Ok(comment_id)
    }
}
