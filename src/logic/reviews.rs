//! Reviews module.

use crate::{
    database::{
        models::{PullRequestModel, ReviewModel},
        DbConn,
    },
    logic::errors::Result,
    types::pull_requests::GHPullRequestReview,
};

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
