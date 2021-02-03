//! Reviews module.

use github_scbot_database::{
    models::{PullRequestModel, ReviewModel},
    DbConn,
};
use github_scbot_types::pull_requests::GHPullRequestReview;

use crate::errors::Result;

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
