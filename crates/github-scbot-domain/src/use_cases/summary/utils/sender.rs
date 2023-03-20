use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{comments::CommentApi, ApiService};
use github_scbot_lock_interface::{LockService, LockStatus};
use tracing::{error, warn};

use super::text_generator::SummaryTextGenerator;
use crate::{use_cases::status::PullRequestStatus, Result};

/// Summary comment sender.
pub struct SummaryCommentSender;

impl SummaryCommentSender {
    /// Creates or updates summary.
    #[tracing::instrument(skip_all, fields(pr_handle), ret)]
    pub async fn create_or_update(
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        lock_service: &dyn LockService,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
    ) -> Result<u64> {
        let comment_id = Self::get_status_comment_id(db_service, pr_handle).await?;
        if comment_id > 0 {
            Self::update(
                api_service,
                db_service,
                pr_handle,
                pull_request_status,
                comment_id,
            )
            .await
        } else {
            // Not the smartest strategy, let's lock with a 10 seconds timeout
            let lock_name = format!(
                "summary-{}-{}-{}",
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number()
            );
            match lock_service.wait_lock_resource(&lock_name, 10_000).await? {
                LockStatus::SuccessfullyLocked(l) => {
                    let comment_id = Self::get_status_comment_id(db_service, pr_handle).await?;
                    let result = if comment_id == 0 {
                        Self::create(api_service, db_service, pr_handle, pull_request_status).await
                    } else {
                        Self::update(
                            api_service,
                            db_service,
                            pr_handle,
                            pull_request_status,
                            comment_id,
                        )
                        .await
                    };

                    l.release().await?;
                    result
                }
                LockStatus::AlreadyLocked => {
                    // Abort summary creation
                    warn!(
                        pr_handle = %pr_handle,
                        message = "Could not create summary after lock timeout. Ignoring."
                    );
                    Ok(0)
                }
            }
        }
    }

    async fn create(
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
    ) -> Result<u64> {
        let status_comment = Self::generate_comment(pull_request_status)?;
        Self::post_github_comment(api_service, db_service, pr_handle, &status_comment).await
    }

    async fn update(
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        pr_handle: &PullRequestHandle,
        pull_request_status: &PullRequestStatus,
        comment_id: u64,
    ) -> Result<u64> {
        let status_comment = Self::generate_comment(pull_request_status)?;

        if let Ok(comment_id) = CommentApi::update_comment(
            api_service,
            pr_handle.repository().owner(),
            pr_handle.repository().name(),
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
            Self::post_github_comment(api_service, db_service, pr_handle, &status_comment).await
        }
    }

    /// Delete comment.
    pub async fn delete(
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        pr_handle: &PullRequestHandle,
    ) -> Result<()> {
        // Re-fetch comment ID
        let comment_id = Self::get_status_comment_id(db_service, pr_handle).await?;

        if comment_id > 0 {
            api_service
                .comments_delete(
                    pr_handle.repository().owner(),
                    pr_handle.repository().name(),
                    comment_id,
                )
                .await?;
        }

        Ok(())
    }

    async fn get_status_comment_id(
        db_service: &dyn DbService,
        pr_handle: &PullRequestHandle,
    ) -> Result<u64> {
        Ok(db_service
            .pull_requests_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
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
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        pr_handle: &PullRequestHandle,
        comment: &str,
    ) -> Result<u64> {
        let comment_id = CommentApi::post_comment(
            api_service,
            pr_handle.repository().owner(),
            pr_handle.repository().name(),
            pr_handle.number(),
            comment,
        )
        .await?;

        db_service
            .pull_requests_set_status_comment_id(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
                comment_id,
            )
            .await?;

        Ok(comment_id)
    }
}
