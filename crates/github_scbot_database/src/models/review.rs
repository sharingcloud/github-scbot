//! Database review models.

use diesel::prelude::*;
use github_scbot_types::reviews::{GHReview, GHReviewState};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::review::{self, dsl},
    DbConn,
};

/// Review model.
#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, AsChangeset)]
#[table_name = "review"]
pub struct ReviewModel {
    /// Database ID.
    pub id: i32,
    /// Pull request database ID.
    pub pull_request_id: i32,
    /// Username.
    pub username: String,
    /// Review state.
    state: String,
    /// Is the review required?
    pub required: bool,
}

/// Review creation.
#[derive(Insertable)]
#[table_name = "review"]
pub struct ReviewCreation<'a> {
    /// Pull request database ID.
    pub pull_request_id: i32,
    /// Username.
    pub username: &'a str,
    /// Review state.
    pub state: String,
    /// Is the review required?
    pub required: bool,
}

impl Default for ReviewCreation<'_> {
    fn default() -> Self {
        Self {
            pull_request_id: 0,
            username: "",
            state: GHReviewState::Pending.to_string(),
            required: false,
        }
    }
}

impl ReviewModel {
    /// Create a review.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Review creation entry
    pub fn create(conn: &DbConn, entry: ReviewCreation) -> Result<Self> {
        diesel::insert_into(dsl::review)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_pull_request_and_username(conn, entry.pull_request_id, &entry.username)
            .ok_or_else(|| {
                DatabaseError::UnknownReviewError(entry.pull_request_id, entry.username.to_string())
            })
    }

    /// Create or update from GitHub review.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request database ID
    /// * `review` - GitHub review
    pub fn create_or_update_from_github_review(
        conn: &DbConn,
        pull_request_id: i32,
        review: &GHReview,
    ) -> Result<Self> {
        let entry = ReviewCreation {
            pull_request_id,
            required: false,
            state: review.state.to_string(),
            username: &review.user.login,
        };

        let mut model = Self::get_or_create(conn, entry)?;
        model.state = review.state.to_string();
        model.save_changes::<Self>(conn)?;

        Ok(model)
    }

    /// Create or update from review state and username.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request database ID
    /// * `review` - GitHub review
    pub fn create_or_update(
        conn: &DbConn,
        pull_request_id: i32,
        review_state: GHReviewState,
        username: &str,
    ) -> Result<Self> {
        let entry = ReviewCreation {
            pull_request_id,
            required: false,
            state: review_state.to_string(),
            username,
        };

        let mut model = Self::get_or_create(conn, entry)?;
        model.state = review_state.to_string();
        model.save_changes::<Self>(conn)?;

        Ok(model)
    }

    /// List reviews.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        review::table.load::<Self>(conn).map_err(Into::into)
    }

    /// List reviews for pull request database ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request database ID
    pub fn list_for_pull_request_id(
        conn: &DbConn,
        pull_request_id: i32,
    ) -> Result<Vec<ReviewModel>> {
        review::table
            .filter(review::pull_request_id.eq(pull_request_id))
            .order_by(review::id)
            .get_results(conn)
            .map_err(Into::into)
    }

    /// Get review for pull request database ID and reviewer username.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request database ID
    /// * `username` - Reviewer username
    pub fn get_from_pull_request_and_username(
        conn: &DbConn,
        pull_request_id: i32,
        username: &str,
    ) -> Option<Self> {
        review::table
            .filter(review::pull_request_id.eq(pull_request_id))
            .filter(review::username.eq(username))
            .first(conn)
            .ok()
    }

    /// Get or create review.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Review creation entry
    pub fn get_or_create(conn: &DbConn, entry: ReviewCreation) -> Result<Self> {
        Self::get_from_pull_request_and_username(conn, entry.pull_request_id, &entry.username)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }

    /// Get review state.
    pub fn get_review_state(&self) -> GHReviewState {
        self.state.as_str().into()
    }

    /// Set review state.
    ///
    /// # Arguments
    ///
    /// * `review_state` - Review state
    pub fn set_review_state(&mut self, review_state: GHReviewState) {
        self.state = review_state.to_string();
    }

    /// Remove review.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove(&self, conn: &DbConn) -> Result<()> {
        diesel::delete(review::table.filter(review::id.eq(self.id))).execute(conn)?;

        Ok(())
    }

    /// Remove reviews for pull request.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request ID
    pub fn remove_all_for_pull_request(conn: &DbConn, pull_request_id: i32) -> Result<()> {
        diesel::delete(review::table.filter(review::pull_request_id.eq(pull_request_id)))
            .execute(conn)?;

        Ok(())
    }

    /// Save model instance to database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)?;

        Ok(())
    }
}
