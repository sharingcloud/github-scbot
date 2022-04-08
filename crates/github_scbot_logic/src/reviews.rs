//! Reviews module.

use std::collections::HashMap;

use github_scbot_conf::Config;
use github_scbot_database::models::{
    HistoryWebhookModel, IDatabaseAdapter, PullRequestModel, RepositoryModel, ReviewModel,
};
use github_scbot_ghapi::{adapter::IAPIAdapter, reviews::ReviewApi};
use github_scbot_redis::{IRedisAdapter, LockStatus};
use github_scbot_types::{
    events::EventType,
    reviews::{GhReview, GhReviewEvent, GhReviewState},
};

use crate::{status::StatusLogic, Result};

/// Handle GitHub pull request review event.
pub async fn handle_review_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhReviewEvent,
) -> Result<()> {
    if let Ok((mut pr, repo)) = db_adapter
        .pull_request()
        .get_from_repository_path_and_number(&event.repository.full_name, event.pull_request.number)
        .await
    {
        if config.server_enable_history_tracking {
            HistoryWebhookModel::builder(&repo, &pr)
                .username(&event.sender.login)
                .event_key(EventType::PullRequestReview)
                .payload(&event)
                .create(db_adapter.history_webhook())
                .await?;
        }

        ReviewLogic::process_review(
            api_adapter,
            db_adapter,
            redis_adapter,
            &repo,
            &pr,
            &event.review,
        )
        .await?;
        StatusLogic::update_pull_request_status(
            api_adapter,
            db_adapter,
            redis_adapter,
            &repo,
            &mut pr,
            &event.pull_request.head.sha,
        )
        .await?;
    }

    Ok(())
}

/// Review logic.
pub struct ReviewLogic;

impl ReviewLogic {
    /// Handle GitHub review.
    #[tracing::instrument(skip(api_adapter, db_adapter, redis_adapter))]
    pub async fn process_review(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        redis_adapter: &dyn IRedisAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        review: &GhReview,
    ) -> Result<()> {
        let permission = api_adapter
            .user_permissions_get(repo_model.owner(), repo_model.name(), &review.user.login)
            .await?;

        let key_name = format!(
            "review_{}-{}_{}_{}",
            repo_model.owner(),
            repo_model.name(),
            pr_model.number(),
            review.user.login
        );
        let timeout = 500;

        if let LockStatus::SuccessfullyLocked(l) =
            redis_adapter.wait_lock_resource(&key_name, timeout).await?
        {
            // Get or create in database
            ReviewModel::builder_from_github(repo_model, pr_model, review)
                .valid(permission.can_write())
                .create_or_update(db_adapter.review())
                .await?;

            l.release().await?;
        };

        Ok(())
    }

    /// Handle review request.
    #[tracing::instrument(skip(api_adapter, db_adapter, redis_adapter))]
    pub async fn process_review_request(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        redis_adapter: &dyn IRedisAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        review_state: GhReviewState,
        requested_reviewers: &[&str],
    ) -> Result<()> {
        for reviewer in requested_reviewers {
            let permission = api_adapter
                .user_permissions_get(repo_model.owner(), repo_model.name(), reviewer)
                .await?;

            let key_name = format!(
                "review_{}-{}_{}_{}",
                repo_model.owner(),
                repo_model.name(),
                pr_model.number(),
                reviewer
            );
            let timeout = 500;

            if let LockStatus::SuccessfullyLocked(l) =
                redis_adapter.wait_lock_resource(&key_name, timeout).await?
            {
                ReviewModel::builder(repo_model, pr_model, reviewer)
                    .state(review_state)
                    .valid(permission.can_write())
                    .create_or_update(db_adapter.review())
                    .await?;

                l.release().await?;
            }
        }

        Ok(())
    }

    /// Re-request existing reviews.
    #[tracing::instrument(skip(api_adapter, db_adapter))]
    pub async fn rerequest_existing_reviews(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
    ) -> Result<()> {
        let reviews = pr_model.reviews(db_adapter.review()).await?;

        if !reviews.is_empty() {
            let reviewers: Vec<_> = reviews.iter().map(|x| x.username().into()).collect();
            api_adapter
                .pull_reviewer_requests_add(
                    repo_model.owner(),
                    repo_model.name(),
                    pr_model.number(),
                    &reviewers,
                )
                .await?;

            for mut review in reviews {
                let update = review
                    .create_update()
                    .state(GhReviewState::Pending)
                    .build_update();
                db_adapter.review().update(&mut review, update).await?;
            }
        }

        Ok(())
    }

    /// Reset reviews.
    pub async fn reset_reviews(
        db_adapter: &dyn IDatabaseAdapter,
        pr_model: &PullRequestModel,
    ) -> Result<()> {
        db_adapter
            .review()
            .remove_all_for_pull_request(pr_model.id())
            .await?;

        Ok(())
    }

    /// Synchronize reviews.
    #[tracing::instrument(skip(api_adapter, db_adapter))]
    pub async fn synchronize_reviews(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
    ) -> Result<()> {
        // Get reviews
        let reviews = ReviewApi::list_reviews_for_pull_request(
            api_adapter,
            repo_model.owner(),
            repo_model.name(),
            pr_model.number(),
        )
        .await?;

        // Update reviews
        let review_map: HashMap<&str, &GhReview> =
            reviews.iter().map(|r| (&r.user.login[..], r)).collect();
        for review in &reviews {
            let permission = api_adapter
                .user_permissions_get(repo_model.owner(), repo_model.name(), &review.user.login)
                .await?;

            ReviewModel::builder(repo_model, pr_model, &review.user.login)
                .state(review.state)
                .valid(permission.can_write())
                .create_or_update(db_adapter.review())
                .await?;
        }

        // Remove missing reviews
        let existing_reviews = pr_model.reviews(db_adapter.review()).await?;
        for review in existing_reviews {
            if !review_map.contains_key(review.username()) {
                db_adapter.review().remove(review).await?;
            }
        }

        Ok(())
    }
}
