//! Database review models.

use github_scbot_types::reviews::{GhReview, GhReviewState};
use serde::{Deserialize, Serialize};

use super::{PullRequestModel, RepositoryModel};
use crate::schema::review;

mod adapter;
mod builder;
pub use adapter::{DummyReviewDbAdapter, IReviewDbAdapter, ReviewDbAdapter};
use builder::ReviewModelBuilder;

/// Review model.
#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    AsChangeset,
    PartialEq,
    Eq,
    Clone,
    Default,
)]
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
pub struct ReviewCreation {
    pub pull_request_id: i32,
    pub username: String,
    pub state: String,
    pub required: bool,
    pub valid: bool,
}

impl From<ReviewModel> for ReviewCreation {
    fn from(model: ReviewModel) -> Self {
        Self {
            pull_request_id: model.pull_request_id,
            username: model.username,
            state: model.state,
            required: model.required,
            valid: model.valid,
        }
    }
}

impl From<ReviewCreation> for ReviewModel {
    fn from(creation: ReviewCreation) -> Self {
        Self {
            id: 0,
            pull_request_id: creation.pull_request_id,
            username: creation.username,
            state: creation.state,
            required: creation.required,
            valid: creation.valid,
        }
    }
}

impl ReviewModel {
    /// Create builder.
    pub fn builder<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        username: &str,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::default(repo_model, pr_model, username)
    }

    /// Create builder from model.
    pub fn builder_from_model<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        model: &Self,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::from_model(repo_model, pr_model, model)
    }

    /// Create builder from GitHub review.
    pub fn builder_from_github<'a>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &GhReview,
    ) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::from_github(repo_model, pr_model, review)
    }

    /// Get review state.
    #[must_use]
    pub fn get_review_state(&self) -> GhReviewState {
        self.state.as_str().into()
    }

    /// Set review state.
    pub fn set_review_state(&mut self, review_state: GhReviewState) {
        self.state = review_state.to_string();
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        models::{DatabaseAdapter, IDatabaseAdapter},
        tests::using_test_db,
        DatabaseError, Result,
    };

    #[actix_rt::test]
    async fn create_and_update() -> Result<()> {
        using_test_db("test_db_review", |config, pool| async move {
            let db_adapter = DatabaseAdapter::new(pool.clone());

            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(db_adapter.repository())
                .await?;

            let pr = PullRequestModel::builder(&repo, 1234, "me")
                .create_or_update(db_adapter.pull_request())
                .await?;

            // Create review
            let mut entry = ReviewModel::builder(&repo, &pr, "him")
                .create_or_update(db_adapter.review())
                .await?;

            assert_eq!(
                entry,
                ReviewModel {
                    id: entry.id,
                    pull_request_id: pr.id(),
                    username: "him".into(),
                    state: GhReviewState::Pending.to_string(),
                    required: false,
                    valid: false
                }
            );

            // Manually update review
            entry.set_review_state(GhReviewState::Commented);
            entry.required = true;
            entry.valid = true;
            db_adapter.review().save(&mut entry).await?;

            // Now, update review with builder
            let entry = ReviewModel::builder(&repo, &pr, "him")
                .required(false)
                .create_or_update(db_adapter.review())
                .await?;

            assert_eq!(
                entry,
                ReviewModel {
                    id: entry.id,
                    pull_request_id: pr.id(),
                    username: "him".into(),
                    state: GhReviewState::Commented.to_string(),
                    required: false,
                    valid: true
                }
            );

            assert_eq!(db_adapter.review().list().await?.len(), 1);

            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
