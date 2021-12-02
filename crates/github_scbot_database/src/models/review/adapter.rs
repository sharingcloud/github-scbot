use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
use tokio_diesel::AsyncRunQueryDsl;

use super::{ReviewCreation, ReviewModel, ReviewUpdate};
use crate::{
    models::{PullRequestModel, RepositoryModel},
    schema::review,
    DatabaseError, DbPool, Result,
};

/// Review DB adapter.
#[async_trait]
pub trait IReviewDbAdapter {
    /// Creates a new review.
    async fn create(&self, entry: ReviewCreation) -> Result<ReviewModel>;
    /// Lists available reviews.
    async fn list(&self) -> Result<Vec<ReviewModel>>;
    /// Lists reviews from pull request ID.
    async fn list_from_pull_request_id(&self, pull_request_id: i32) -> Result<Vec<ReviewModel>>;
    /// Lists reviews from pull request and username.
    async fn get_from_pull_request_and_username(
        &self,
        repository: &RepositoryModel,
        pull_request: &PullRequestModel,
        username: &str,
    ) -> Result<ReviewModel>;
    /// Removes an existing review.
    async fn remove(&self, entry: ReviewModel) -> Result<()>;
    /// Removes all existing reviews for a pull request ID.
    async fn remove_all_for_pull_request(&self, pull_request_id: i32) -> Result<()>;
    /// Update.
    async fn update(&self, entry: &mut ReviewModel, update: ReviewUpdate) -> Result<()>;
}

/// Concrete review DB adapter.
pub struct ReviewDbAdapter {
    pool: DbPool,
}

impl ReviewDbAdapter {
    /// Creates a new review DB adapter.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IReviewDbAdapter for ReviewDbAdapter {
    async fn create(&self, entry: ReviewCreation) -> Result<ReviewModel> {
        diesel::insert_into(review::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list(&self) -> Result<Vec<ReviewModel>> {
        review::table
            .load_async::<ReviewModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list_from_pull_request_id(&self, pull_request_id: i32) -> Result<Vec<ReviewModel>> {
        review::table
            .filter(review::pull_request_id.eq(pull_request_id))
            .order_by(review::id)
            .get_results_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn get_from_pull_request_and_username(
        &self,
        repository: &RepositoryModel,
        pull_request: &PullRequestModel,
        username: &str,
    ) -> Result<ReviewModel> {
        let username = username.to_owned();
        let repository = repository.clone();
        let pull_request = pull_request.clone();

        review::table
            .filter(review::pull_request_id.eq(pull_request.id()))
            .filter(review::username.eq(username.clone()))
            .first_async(&self.pool)
            .await
            .map_err(|_e| {
                DatabaseError::UnknownReviewState(
                    username.to_string(),
                    repository.path(),
                    pull_request.number(),
                )
            })
    }

    async fn remove(&self, entry: ReviewModel) -> Result<()> {
        diesel::delete(review::table.filter(review::id.eq(entry.id)))
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }

    async fn remove_all_for_pull_request(&self, pull_request_id: i32) -> Result<()> {
        diesel::delete(review::table.filter(review::pull_request_id.eq(pull_request_id)))
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, entry: &mut ReviewModel, update: ReviewUpdate) -> Result<()> {
        *entry = diesel::update(review::table.filter(review::id.eq(entry.id)))
            .set(update)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy review DB adapter.
pub struct DummyReviewDbAdapter {
    /// Create response.
    pub create_response: Mock<ReviewCreation, Result<ReviewModel>>,
    /// List response.
    pub list_response: Mock<(), Result<Vec<ReviewModel>>>,
    /// List from pull request ID response.
    pub list_from_pull_request_id_response: Mock<i32, Result<Vec<ReviewModel>>>,
    /// Get from pull request and username response.
    pub get_from_pull_request_and_username_response:
        Mock<(RepositoryModel, PullRequestModel, String), Result<ReviewModel>>,
    /// Remove response.
    pub remove_response: Mock<ReviewModel, Result<()>>,
    /// Remove all for pull request response.
    pub remove_all_for_pull_request_response: Mock<i32, Result<()>>,
}

impl Default for DummyReviewDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(Box::new(|e| Ok(e.into()))),
            list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            list_from_pull_request_id_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            get_from_pull_request_and_username_response: Mock::new(Box::new(|_| {
                Ok(ReviewModel::default())
            })),
            remove_response: Mock::new(Box::new(|_| Ok(()))),
            remove_all_for_pull_request_response: Mock::new(Box::new(|_| Ok(()))),
        }
    }
}

impl DummyReviewDbAdapter {
    /// Creates a dummy review DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IReviewDbAdapter for DummyReviewDbAdapter {
    async fn create(&self, entry: ReviewCreation) -> Result<ReviewModel> {
        self.create_response.call(entry)
    }

    async fn list(&self) -> Result<Vec<ReviewModel>> {
        self.list_response.call(())
    }

    async fn list_from_pull_request_id(&self, pull_request_id: i32) -> Result<Vec<ReviewModel>> {
        self.list_from_pull_request_id_response
            .call(pull_request_id)
    }

    async fn get_from_pull_request_and_username(
        &self,
        repository: &RepositoryModel,
        pull_request: &PullRequestModel,
        username: &str,
    ) -> Result<ReviewModel> {
        self.get_from_pull_request_and_username_response.call((
            repository.clone(),
            pull_request.clone(),
            username.to_owned(),
        ))
    }

    async fn remove(&self, entry: ReviewModel) -> Result<()> {
        self.remove_response.call(entry)
    }

    async fn remove_all_for_pull_request(&self, pull_request_id: i32) -> Result<()> {
        self.remove_all_for_pull_request_response
            .call(pull_request_id)
    }

    async fn update(&self, entry: &mut ReviewModel, update: ReviewUpdate) -> Result<()> {
        entry.apply_local_update(update);
        Ok(())
    }
}
