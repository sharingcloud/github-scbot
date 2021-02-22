//! Database review models.

use diesel::prelude::*;
use github_scbot_types::reviews::{GHReview, GHReviewState};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{DatabaseError, Result},
    schema::review,
    DbConn,
};

use super::{PullRequestModel, RepositoryModel};

/// Review model.
#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, AsChangeset, PartialEq, Eq)]
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
    /// Is the review valid?
    pub valid: bool,
}

#[derive(Insertable)]
#[table_name = "review"]
struct ReviewCreation {
    pub pull_request_id: i32,
    pub username: String,
    pub state: String,
    pub required: bool,
    pub valid: bool,
}

impl From<&ReviewModel> for ReviewCreation {
    fn from(model: &ReviewModel) -> Self {
        Self {
            pull_request_id: model.pull_request_id,
            username: model.username.clone(),
            state: model.state.clone(),
            required: model.required,
            valid: model.valid,
        }
    }
}

#[must_use]
pub struct ReviewModelBuilder<'a> {
    repo_model: &'a RepositoryModel,
    pr_model: &'a PullRequestModel,
    username: String,
    state: Option<GHReviewState>,
    required: Option<bool>,
    valid: Option<bool>,
}

impl<'a> ReviewModelBuilder<'a> {
    pub fn default<T: Into<String>>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        username: T,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: username.into(),
            state: None,
            required: None,
            valid: None,
        }
    }

    pub fn from_model(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &ReviewModel,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: review.username.clone(),
            state: Some(review.get_review_state()),
            required: Some(review.required),
            valid: Some(review.valid),
        }
    }

    pub fn from_github(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &GHReview,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: review.user.login.clone(),
            state: Some(review.state),
            required: None,
            valid: None,
        }
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.username = username.into();
        self
    }

    pub fn state<T: Into<GHReviewState>>(mut self, state: T) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn required<T: Into<bool>>(mut self, required: T) -> Self {
        self.required = Some(required.into());
        self
    }

    pub fn valid<T: Into<bool>>(mut self, valid: T) -> Self {
        self.valid = Some(valid.into());
        self
    }

    fn build(&self) -> ReviewModel {
        ReviewModel {
            id: -1,
            pull_request_id: self.pr_model.id,
            username: self.username.clone(),
            state: self.state.unwrap_or(GHReviewState::Pending).to_string(),
            required: self.required.unwrap_or(false),
            valid: self.valid.unwrap_or(false),
        }
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<ReviewModel> {
        let mut handle = match ReviewModel::get_from_pull_request_and_username(
            conn,
            self.repo_model,
            self.pr_model,
            &self.username,
        ) {
            Ok(entry) => entry,
            Err(_) => {
                let entry = self.build();
                ReviewModel::create(conn, (&entry).into())?
            }
        };

        handle.state = match self.state {
            Some(s) => s.to_string(),
            None => handle.state,
        };
        handle.required = match self.required {
            Some(r) => r,
            None => handle.required,
        };
        handle.valid = match self.valid {
            Some(v) => v,
            None => handle.valid,
        };
        handle.save(conn)?;

        Ok(handle)
    }
}

impl ReviewModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `pr_model` - Pull request
    /// * `username` - Username
    pub fn builder<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        username: &str,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::default(repo_model, pr_model, username)
    }

    /// Create builder from model.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `pr_model` - Pull request
    /// * `model` - Model
    pub fn builder_from_model<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        model: &Self,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::from_model(repo_model, pr_model, model)
    }

    /// Create builder from GitHub review.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository
    /// * `pr_model` - Pull request
    /// * `review` - Review
    pub fn builder_from_github<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &GHReview,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::from_github(repo_model, pr_model, review)
    }

    fn create(conn: &DbConn, entry: ReviewCreation) -> Result<Self> {
        diesel::insert_into(review::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// List reviews.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        review::table.load::<Self>(conn).map_err(Into::into)
    }

    /// List reviews from pull request database ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `pull_request_id` - Pull request database ID
    pub fn list_from_pull_request_id(
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
        repository: &RepositoryModel,
        pull_request: &PullRequestModel,
        username: &str,
    ) -> Result<Self> {
        review::table
            .filter(review::pull_request_id.eq(pull_request.id))
            .filter(review::username.eq(username))
            .first(conn)
            .map_err(|_e| {
                DatabaseError::UnknownReviewState(
                    username.to_string(),
                    repository.get_path(),
                    pull_request.get_number(),
                )
            })
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

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use pretty_assertions::assert_eq;

    use crate::establish_single_test_connection;

    use super::*;

    fn test_init() -> (Config, DbConn) {
        let config = Config::from_env();
        let conn = establish_single_test_connection(&config).unwrap();

        (config, conn)
    }

    #[test]
    fn create_and_update() {
        let (config, conn) = test_init();

        let repo = RepositoryModel::builder(&config, "me", "TestRepo")
            .create_or_update(&conn)
            .unwrap();

        let pr = PullRequestModel::builder(&repo, 1234, "me")
            .create_or_update(&conn)
            .unwrap();

        // Create review
        let mut entry = ReviewModel::builder(&repo, &pr, "him")
            .create_or_update(&conn)
            .unwrap();

        assert_eq!(
            entry,
            ReviewModel {
                id: entry.id,
                pull_request_id: pr.id,
                username: "him".into(),
                state: GHReviewState::Pending.to_string(),
                required: false,
                valid: false
            }
        );

        // Manually update review
        entry.set_review_state(GHReviewState::Commented);
        entry.required = true;
        entry.valid = true;
        entry.save(&conn).unwrap();

        // Now, update review with builder
        let entry = ReviewModel::builder(&repo, &pr, "him")
            .required(false)
            .create_or_update(&conn)
            .unwrap();

        assert_eq!(
            entry,
            ReviewModel {
                id: entry.id,
                pull_request_id: pr.id,
                username: "him".into(),
                state: GHReviewState::Commented.to_string(),
                required: false,
                valid: true
            }
        );

        assert_eq!(ReviewModel::list(&conn).unwrap().len(), 1);
    }
}
