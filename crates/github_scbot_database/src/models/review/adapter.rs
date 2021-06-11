use async_trait::async_trait;
use diesel::prelude::*;
use tokio_diesel::AsyncRunQueryDsl;

use super::{ReviewCreation, ReviewModel};
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
    /// Saves and updates an existing review.
    async fn save(&self, entry: &mut ReviewModel) -> Result<()>;
}

/// Concrete review DB adapter.
pub struct ReviewDbAdapter<'a> {
    pool: &'a DbPool,
}

impl<'a> ReviewDbAdapter<'a> {
    /// Creates a new review DB adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IReviewDbAdapter for ReviewDbAdapter<'a> {
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
            .filter(review::pull_request_id.eq(pull_request.id))
            .filter(review::username.eq(username.clone()))
            .first_async(&self.pool)
            .await
            .map_err(|_e| {
                DatabaseError::UnknownReviewState(
                    username.to_string(),
                    repository.get_path(),
                    pull_request.get_number(),
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

    async fn save(&self, entry: &mut ReviewModel) -> Result<()> {
        let copy = entry.clone();

        *entry = diesel::update(review::table.filter(review::id.eq(copy.id)))
            .set(copy)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy review DB adapter.
pub struct DummyReviewDbAdapter {
    /// Create response.
    pub create_response: Result<ReviewModel>,
    /// List response.
    pub list_response: Result<Vec<ReviewModel>>,
    /// List from pull request ID response.
    pub list_from_pull_request_id_response: Result<Vec<ReviewModel>>,
    /// Get from pull request and username response.
    pub get_from_pull_request_and_username_response: Result<ReviewModel>,
    /// Remove response.
    pub remove_response: Result<()>,
    /// Remove all for pull request response.
    pub remove_all_for_pull_request_response: Result<()>,
    /// Save response.
    pub save_response: Result<()>,
}

impl Default for DummyReviewDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Ok(ReviewModel::default()),
            list_response: Ok(Vec::new()),
            list_from_pull_request_id_response: Ok(Vec::new()),
            get_from_pull_request_and_username_response: Ok(ReviewModel::default()),
            remove_response: Ok(()),
            remove_all_for_pull_request_response: Ok(()),
            save_response: Ok(()),
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
        self.create_response.clone()
    }

    async fn list(&self) -> Result<Vec<ReviewModel>> {
        self.list_response.clone()
    }

    async fn list_from_pull_request_id(&self, pull_request_id: i32) -> Result<Vec<ReviewModel>> {
        self.list_from_pull_request_id_response.clone()
    }

    async fn get_from_pull_request_and_username(
        &self,
        repository: &RepositoryModel,
        pull_request: &PullRequestModel,
        username: &str,
    ) -> Result<ReviewModel> {
        self.get_from_pull_request_and_username_response.clone()
    }

    async fn remove(&self, entry: ReviewModel) -> Result<()> {
        self.remove_response.clone()
    }

    async fn remove_all_for_pull_request(&self, pull_request_id: i32) -> Result<()> {
        self.remove_all_for_pull_request_response.clone()
    }

    async fn save(&self, entry: &mut ReviewModel) -> Result<()> {
        self.save_response.clone()
    }
}