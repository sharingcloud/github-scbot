//! Reviews module.

use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::{
    HistoryWebhookModel, IDatabaseAdapter, PullRequestModel, RepositoryModel, ReviewModel,
};
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{
    events::EventType,
    reviews::{GhReview, GhReviewEvent, GhReviewState},
};

use crate::{status::update_pull_request_status, Result};

/// Handle GitHub pull request review event.
pub async fn handle_review_event(
    config: &Config,
    api_adapter: &impl IAPIAdapter,
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

        process_review(api_adapter, db_adapter, &repo, &pr, &event.review).await?;
        update_pull_request_status(
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

/// Handle GitHub review.
pub async fn process_review(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review: &GhReview,
) -> Result<()> {
    let permission = api_adapter
        .user_permissions_get(&repo_model.owner, &repo_model.name, &review.user.login)
        .await?;

    // Get or create in database
    ReviewModel::builder_from_github(&repo_model, &pr_model, review)
        .valid(permission.can_write())
        .create_or_update(db_adapter.review())
        .await?;

    Ok(())
}

/// Handle review request.
pub async fn process_review_request(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review_state: GhReviewState,
    requested_reviewers: &[&str],
) -> Result<()> {
    for reviewer in requested_reviewers {
        let permission = api_adapter
            .user_permissions_get(&repo_model.owner, &repo_model.name, reviewer)
            .await?;

        ReviewModel::builder(repo_model, pr_model, reviewer)
            .state(review_state)
            .valid(permission.can_write())
            .create_or_update(db_adapter.review())
            .await?;
    }

    Ok(())
}

/// Re-request existing reviews.
pub async fn rerequest_existing_reviews(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    let reviews = pr_model.get_reviews(db_adapter.review()).await?;

    if !reviews.is_empty() {
        let reviewers: Vec<_> = reviews.iter().map(|x| x.username.clone()).collect();
        api_adapter
            .pull_reviewer_requests_add(
                &repo_model.owner,
                &repo_model.name,
                pr_model.get_number(),
                &reviewers,
            )
            .await?;

        for mut review in reviews {
            review.set_review_state(GhReviewState::Pending);
            db_adapter.review().save(&mut review).await?;
        }
    }

    Ok(())
}
