use prbot_ghapi_interface::comments::CommentApi;
use prbot_lock_interface::{using_lock, UsingLockResult};
use prbot_models::PullRequestHandle;
use tracing::{error, warn};

use super::text_generator::SummaryTextGenerator;
use crate::{use_cases::status::PullRequestStatus, CoreContext, Result};

/// Summary comment sender.
pub struct SummaryCommentSender;

impl SummaryCommentSender {
    /// Creates or updates summary.
    #[tracing::instrument(skip_all, fields(pr_handle), ret)]
    pub async fn create_or_update(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
    ) -> Result<u64> {
        let comment_id = Self::get_status_comment_id(ctx, pr_handle).await?;
        if comment_id > 0 {
            Self::update(ctx, pr_handle, pull_request_status, comment_id).await
        } else {
            // Not the smartest strategy, let's lock with a 10 seconds timeout
            let lock_name = format!(
                "summary-{}-{}-{}",
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number()
            );

            let output = using_lock(ctx.lock_service, &lock_name, 10_000, || async {
                let comment_id = Self::get_status_comment_id(ctx, pr_handle).await?;
                if comment_id == 0 {
                    Self::create(ctx, pr_handle, pull_request_status).await
                } else {
                    Self::update(ctx, pr_handle, pull_request_status, comment_id).await
                }
            })
            .await?;

            match output {
                UsingLockResult::AlreadyLocked => {
                    // Abort summary creation
                    warn!(
                        pr_handle = %pr_handle,
                        message = "Could not create summary after lock timeout. Ignoring."
                    );
                    Ok(0)
                }
                UsingLockResult::Locked(result) => result,
            }
        }
    }

    async fn create(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
    ) -> Result<u64> {
        let status_comment = Self::generate_comment(pull_request_status)?;
        Self::post_github_comment(ctx, pr_handle, &status_comment).await
    }

    async fn update(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
        comment_id: u64,
    ) -> Result<u64> {
        let status_comment = Self::generate_comment(pull_request_status)?;

        if let Ok(comment_id) = CommentApi::update_comment(
            ctx.config,
            ctx.api_service,
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            comment_id,
            &status_comment,
        )
        .await
        {
            Ok(comment_id)
        } else {
            error!(
                comment_id = comment_id,
                pr_handle = %pr_handle,
                message = "Comment ID is not valid anymore, will post another status comment"
            );

            // Comment ID is no more used on GitHub, recreate a new status
            Self::post_github_comment(ctx, pr_handle, &status_comment).await
        }
    }

    /// Delete comment.
    pub async fn delete(ctx: &CoreContext<'_>, pr_handle: &PullRequestHandle) -> Result<()> {
        // Re-fetch comment ID
        let comment_id = Self::get_status_comment_id(ctx, pr_handle).await?;

        if comment_id > 0 {
            ctx.api_service
                .comments_delete(
                    pr_handle.repository_path().owner(),
                    pr_handle.repository_path().name(),
                    comment_id,
                )
                .await?;
        }

        Ok(())
    }

    async fn get_status_comment_id(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
    ) -> Result<u64> {
        Ok(ctx
            .db_service
            .pull_requests_get(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?
            .map(|pr| pr.status_comment_id)
            .unwrap_or(0))
    }

    fn generate_comment(pull_request_status: &PullRequestStatus) -> Result<String> {
        SummaryTextGenerator::generate(pull_request_status)
    }

    async fn post_github_comment(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        comment: &str,
    ) -> Result<u64> {
        let comment_id = CommentApi::post_comment(
            ctx.config,
            ctx.api_service,
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            pr_handle.number(),
            comment,
        )
        .await?;

        ctx.db_service
            .pull_requests_set_status_comment_id(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
                comment_id,
            )
            .await?;

        Ok(comment_id)
    }
}
