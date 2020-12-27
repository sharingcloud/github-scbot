//! Reviews logic

use crate::database::{
    models::{PullRequestModel, ReviewModel},
    DbConn,
};
use crate::types::PullRequestReview;
use crate::webhook::errors::Result;

pub async fn handle_review(
    conn: &DbConn,
    pr_model: &PullRequestModel,
    review: &PullRequestReview,
) -> Result<()> {
    // Get or create in database
    ReviewModel::create_or_update_from_review(conn, pr_model.id, review).unwrap();

    Ok(())
}
