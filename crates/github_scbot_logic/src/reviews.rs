//! Reviews module.

use github_scbot_api::{
    auth::get_user_permission_on_repository, reviews::request_reviewers_for_pull_request,
};
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{HistoryWebhookModel, PullRequestModel, RepositoryModel, ReviewModel},
    DbConn, DbPool,
};
use github_scbot_types::{
    events::EventType,
    reviews::{GhReview, GhReviewEvent, GhReviewState},
};

use crate::{status::update_pull_request_status, Result};

/// Handle GitHub pull request review event.
pub async fn handle_review_event(config: Config, pool: DbPool, event: GhReviewEvent) -> Result<()> {
    let conn = get_connection(&pool)?;

    if let Ok((mut pr, repo)) = PullRequestModel::get_from_repository_path_and_number(
        &conn,
        &event.repository.full_name,
        event.pull_request.number,
    ) {
        HistoryWebhookModel::builder(&repo, &pr)
            .username(&event.sender.login)
            .event_key(EventType::PullRequestReview)
            .payload(&event)
            .create(&conn)?;

        handle_review(&config, &conn, &repo, &pr, &event.review).await?;
        update_pull_request_status(
            &config,
            pool.clone(),
            &repo,
            &mut pr,
            &event.pull_request.head.sha,
        )
        .await?;
    }

    Ok(())
}

/// Handle GitHub review.
pub async fn handle_review(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review: &GhReview,
) -> Result<()> {
    let permission = get_user_permission_on_repository(
        config,
        &repo_model.owner,
        &repo_model.name,
        &review.user.login,
    )
    .await?;

    // Get or create in database
    ReviewModel::builder_from_github(&repo_model, &pr_model, review)
        .valid(permission.can_write())
        .create_or_update(conn)?;

    Ok(())
}

/// Handle review request.
pub async fn handle_review_request(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review_state: GhReviewState,
    requested_reviewers: &[&str],
) -> Result<()> {
    for reviewer in requested_reviewers {
        let permission = get_user_permission_on_repository(
            config,
            &repo_model.owner,
            &repo_model.name,
            reviewer,
        )
        .await?;

        ReviewModel::builder(repo_model, pr_model, reviewer)
            .state(review_state)
            .valid(permission.can_write())
            .create_or_update(conn)?;
    }

    Ok(())
}

/// Re-request existing reviews.
pub async fn rerequest_existing_reviews(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    let reviews = pr_model.get_reviews(conn)?;

    if !reviews.is_empty() {
        let reviewers: Vec<_> = reviews.iter().map(|x| x.username.clone()).collect();
        request_reviewers_for_pull_request(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &reviewers,
        )
        .await?;

        for mut review in reviews {
            review.set_review_state(GhReviewState::Pending);
            review.save(conn)?;
        }
    }

    Ok(())
}
