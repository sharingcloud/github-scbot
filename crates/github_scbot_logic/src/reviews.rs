//! Reviews module.

use github_scbot_database::{
    models::{PullRequestModel, ReviewModel},
    DbConn,
};
use github_scbot_types::pull_requests::{GHPullRequestReview, GHPullRequestReviewEvent};

use crate::{database::process_pull_request, status::update_pull_request_status, Result};

/// Handle GitHub pull request review event.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `event` - GitHub pull request review event
pub async fn handle_review_event(conn: &DbConn, event: &GHPullRequestReviewEvent) -> Result<()> {
    let (repo, mut pr) = process_pull_request(conn, &event.repository, &event.pull_request)?;
    handle_review(conn, &pr, &event.review).await?;
    update_pull_request_status(conn, &repo, &mut pr, &event.pull_request.head.sha).await?;

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
    conn: &DbConn,
    pr_model: &PullRequestModel,
    review: &GHPullRequestReview,
) -> Result<()> {
    // Get or create in database
    ReviewModel::create_or_update_from_github_review(conn, pr_model.id, review)?;

    Ok(())
}
