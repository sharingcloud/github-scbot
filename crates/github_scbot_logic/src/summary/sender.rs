use github_scbot_database2::DbService;
use github_scbot_ghapi::{adapter::ApiService, comments::CommentApi};
use github_scbot_types::pulls::GhPullRequest;
use tracing::warn;

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
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<u64> {
        let pull_request_status = PullRequestStatus::from_database(
            api_adapter,
            db_adapter,
            repo_owner,
            repo_name,
            pr_number,
            upstream_pr,
        )
        .await?;
        let status_comment = Self::generate_comment(&pull_request_status)?;
        self.post_github_comment(
            api_adapter,
            db_adapter,
            repo_owner,
            repo_name,
            pr_number,
            &status_comment,
        )
        .await
    }

    /// Update comment.
    pub async fn update(
        &self,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<u64> {
        let pull_request_status = PullRequestStatus::from_database(
            api_adapter,
            db_adapter,
            repo_owner,
            repo_name,
            pr_number,
            upstream_pr,
        )
        .await?;
        let status_comment = Self::generate_comment(&pull_request_status)?;

        let comment_id =
            Self::get_status_comment_id(db_adapter, repo_owner, repo_name, pr_number).await?;
        if comment_id > 0 {
            if let Ok(comment_id) = CommentApi::update_comment(
                api_adapter,
                repo_owner,
                repo_name,
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
                    repo_owner,
                    repo_name,
                    pr_number,
                    &status_comment,
                )
                .await
            }
        } else {
            // Too early, do not update the status comment
            warn!(
                status = ?pull_request_status,
                message = "Could not update summary, comment ID is 0"
            );

            Ok(0)
        }
    }

    /// Delete comment.
    pub async fn delete(
        &self,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<()> {
        // Re-fetch comment ID
        let comment_id =
            Self::get_status_comment_id(db_adapter, repo_owner, repo_name, pr_number).await?;

        if comment_id > 0 {
            api_adapter
                .comments_delete(repo_owner, repo_name, comment_id)
                .await?;
        }

        Ok(())
    }

    async fn get_status_comment_id(
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<u64> {
        Ok(db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
            .await?
            .map(|pr| pr.status_comment_id())
            .unwrap_or(0))
    }

    fn generate_comment(pull_request_status: &PullRequestStatus) -> Result<String> {
        SummaryTextGenerator::generate(pull_request_status)
    }

    async fn post_github_comment(
        &self,
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        issue_number: u64,
        comment: &str,
    ) -> Result<u64> {
        let comment_id =
            CommentApi::post_comment(api_adapter, repo_owner, repo_name, issue_number, comment)
                .await?;

        db_adapter
            .pull_requests()
            .set_status_comment_id(repo_owner, repo_name, issue_number, comment_id)
            .await?;

        Ok(comment_id)
    }
}
