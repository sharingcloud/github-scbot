//! Database review models.

use github_scbot_database_macros::SCGetter;
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
    Debug, Deserialize, Serialize, Queryable, Identifiable, PartialEq, Eq, Clone, Default, SCGetter,
)]
#[table_name = "review"]
pub struct ReviewModel {
    /// Database ID.
    #[get]
    id: i32,
    /// Pull request database ID.
    #[get]
    pull_request_id: i32,
    /// Username.
    #[get_ref]
    username: String,
    /// Review state.
    state: String,
    /// Is the review required?
    #[get]
    required: bool,
    /// Is the review valid?
    #[get]
    valid: bool,
    /// Has approved?
    #[get]
    approved: bool,
}

#[derive(Debug, Identifiable, Clone, AsChangeset, Default)]
#[table_name = "review"]
pub struct ReviewUpdate {
    /// Database ID.
    pub id: i32,
    /// Username.
    pub username: Option<String>,
    /// Review state.
    pub state: Option<String>,
    /// Review required?
    pub required: Option<bool>,
    /// Review valid?
    pub valid: Option<bool>,
    /// Has approved?
    pub approved: Option<bool>,
}

#[derive(Insertable)]
#[table_name = "review"]
pub struct ReviewCreation {
    pub pull_request_id: i32,
    pub username: String,
    pub state: String,
    pub required: bool,
    pub valid: bool,
    pub approved: bool,
}

impl From<ReviewModel> for ReviewCreation {
    fn from(model: ReviewModel) -> Self {
        Self {
            pull_request_id: model.pull_request_id,
            username: model.username,
            state: model.state,
            required: model.required,
            valid: model.valid,
            approved: model.approved,
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
            approved: creation.approved,
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
        ReviewModelBuilder::new(repo_model, pr_model, username)
    }

    /// Prepare an update builder.
    pub fn create_update<'a>(&self) -> ReviewModelBuilder<'a> {
        ReviewModelBuilder::with_id(self.id)
    }

    /// Apply local update on pull request.
    /// Result will not be in database.
    pub fn apply_local_update(&mut self, update: ReviewUpdate) {
        if let Some(s) = update.state {
            self.state = s;
        }

        if let Some(s) = update.required {
            self.required = s;
        }

        if let Some(s) = update.valid {
            self.valid = s;
        }
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
    pub fn state(&self) -> GhReviewState {
        self.state.as_str().into()
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
                    valid: false,
                    approved: false,
                }
            );

            // Manually update review
            let update = entry
                .create_update()
                .state(GhReviewState::Commented)
                .required(true)
                .valid(true)
                .build_update();
            db_adapter.review().update(&mut entry, update).await?;

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
                    valid: true,
                    approved: false,
                }
            );

            assert_eq!(db_adapter.review().list().await?.len(), 1);

            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
