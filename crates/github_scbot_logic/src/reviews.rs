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
    reviews::{GHReview, GHReviewEvent, GHReviewState},
};

use crate::{database::process_pull_request, status::update_pull_request_status, Result};

/// Handle GitHub pull request review event.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `pool` - Database pool
/// * `event` - GitHub pull request review event
pub async fn handle_review_event(config: Config, pool: DbPool, event: GHReviewEvent) -> Result<()> {
    let (repo, mut pr) = process_pull_request(
        config.clone(),
        pool.clone(),
        event.repository.clone(),
        event.pull_request.clone(),
    )
    .await?;

    let conn = get_connection(&pool)?;
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

    Ok(())
}

/// Handle GitHub review.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pr_model` - Pull request model
/// * `review` - GitHub review
pub async fn handle_review(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review: &GHReview,
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
///
/// # Arguments
///
/// * `conn` - Database connection
pub async fn handle_review_request(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    review_state: GHReviewState,
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
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` -Repository model
/// * `pr_model` - Pull request model
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
            review.set_review_state(GHReviewState::Pending);
            review.save(conn)?;
        }
    }

    Ok(())
}
