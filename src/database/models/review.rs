//! Database review models

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::DbConn;
use crate::database::schema::review::{self, dsl};
use crate::types::PullRequestReviewState;
use crate::{
    database::errors::{DatabaseError, Result},
    types::PullRequestReview,
};

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Identifiable, AsChangeset)]
#[table_name = "review"]
pub struct ReviewModel {
    pub id: i32,
    pub pull_request_id: i32,
    pub username: String,
    pub state: String,
    pub required: bool,
}

#[derive(Insertable)]
#[table_name = "review"]
pub struct ReviewCreation<'a> {
    pub pull_request_id: i32,
    pub username: &'a str,
    pub state: String,
    pub required: bool,
}

impl Default for ReviewCreation<'_> {
    fn default() -> Self {
        Self {
            pull_request_id: 0,
            username: "",
            state: PullRequestReviewState::Pending.to_string(),
            required: false,
        }
    }
}

impl ReviewModel {
    pub fn state_enum(&self) -> PullRequestReviewState {
        self.state.as_str().into()
    }

    pub fn update_from_instance(&mut self, conn: &DbConn, other: &Self) -> Result<()> {
        self.username = other.username.clone();
        self.state = other.state.clone();
        self.required = other.required;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        review::table.load::<Self>(conn).map_err(Into::into)
    }

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

    pub fn update_required(&mut self, conn: &DbConn, value: bool) -> Result<()> {
        self.required = value;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    #[allow(clippy::clippy::needless_pass_by_value)]
    pub fn create(conn: &DbConn, entry: ReviewCreation) -> Result<Self> {
        diesel::insert_into(dsl::review)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_pull_request_and_username(conn, entry.pull_request_id, &entry.username)
            .ok_or_else(|| {
                DatabaseError::UnknownReviewError(entry.pull_request_id, entry.username.to_string())
            })
    }

    pub fn create_or_update_from_review(
        conn: &DbConn,
        pull_request_id: i32,
        review: &PullRequestReview,
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

    pub fn get_or_create(conn: &DbConn, entry: ReviewCreation) -> Result<Self> {
        Self::get_from_pull_request_and_username(conn, entry.pull_request_id, &entry.username)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }
}
