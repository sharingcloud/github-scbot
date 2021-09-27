use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
use tokio_diesel::AsyncRunQueryDsl;

use super::{PullRequestCreation, PullRequestModel};
use crate::{
    models::RepositoryModel,
    schema::{pull_request, repository, review},
    DatabaseError, DbPool, Result,
};

/// Pull request DB adapter.
#[async_trait]
pub trait IPullRequestDbAdapter {
    /// Creates a pull request.
    async fn create(&self, entry: PullRequestCreation) -> Result<PullRequestModel>;
    /// Fetch status comment ID from a pull request ID.
    async fn fetch_status_comment_id(&self, pull_request_id: i32) -> Result<i32>;
    /// Lists available pull requests.
    async fn list(&self) -> Result<Vec<PullRequestModel>>;
    /// Lists available pull requests from a repository path.
    async fn list_from_repository_path(&self, path: &str) -> Result<Vec<PullRequestModel>>;
    /// Gets an existing pull request from a repository and a pull request number.
    async fn get_from_repository_and_number(
        &self,
        repository: &RepositoryModel,
        number: u64,
    ) -> Result<PullRequestModel>;
    /// Gets an existing pull request from a repository path and a pull request number.
    async fn get_from_repository_path_and_number(
        &self,
        path: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)>;
    /// Lists closed pull requests from a repository.
    async fn list_closed_pulls_from_repository(
        &self,
        repository_id: i32,
    ) -> Result<Vec<PullRequestModel>>;
    /// Removes an existing pull request.
    async fn remove(&self, entry: &PullRequestModel) -> Result<()>;
    /// Removes closed pull requests from a repository.
    async fn remove_closed_pulls_from_repository(&self, repository_id: i32) -> Result<()>;
    /// Saves and updates a pull request.
    async fn save(&self, entry: &mut PullRequestModel) -> Result<()>;
}

/// Concrete pull request DB adapter.
pub struct PullRequestDbAdapter {
    pool: DbPool,
}

impl PullRequestDbAdapter {
    /// Creates a new pull request DB adapter.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IPullRequestDbAdapter for PullRequestDbAdapter {
    async fn create(&self, entry: PullRequestCreation) -> Result<PullRequestModel> {
        diesel::insert_into(pull_request::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn fetch_status_comment_id(&self, pull_request_id: i32) -> Result<i32> {
        pull_request::table
            .filter(pull_request::id.eq(pull_request_id))
            .select(pull_request::status_comment_id)
            .get_result_async::<i32>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list(&self) -> Result<Vec<PullRequestModel>> {
        pull_request::table
            .load_async::<PullRequestModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list_from_repository_path(&self, path: &str) -> Result<Vec<PullRequestModel>> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;
        let owner = owner.to_owned();
        let name = name.to_owned();

        let values: Vec<(PullRequestModel, Option<RepositoryModel>)> = pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .get_results_async(&self.pool)
            .await?;

        Ok(values.into_iter().map(|(pr, _repo)| pr).collect())
    }

    async fn get_from_repository_and_number(
        &self,
        repository: &RepositoryModel,
        number: u64,
    ) -> Result<PullRequestModel> {
        let repository = repository.clone();

        pull_request::table
            .filter(pull_request::repository_id.eq(repository.id))
            .filter(pull_request::number.eq(number as i32))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownPullRequest(repository.get_path(), number))
    }

    async fn get_from_repository_path_and_number(
        &self,
        path: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;
        let owner = owner.to_owned();
        let name = name.to_owned();
        let path = path.to_owned();

        let (pr, repo): (PullRequestModel, Option<RepositoryModel>) = pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .filter(pull_request::number.eq(number as i32))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownPullRequest(path.to_string(), number))?;

        Ok((
            pr,
            repo.expect("Invalid repository linked to pull request."),
        ))
    }

    async fn list_closed_pulls_from_repository(
        &self,
        repository_id: i32,
    ) -> Result<Vec<PullRequestModel>> {
        pull_request::table
            .filter(pull_request::repository_id.eq(repository_id))
            .filter(pull_request::closed.eq(true))
            .get_results_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn remove(&self, entry: &PullRequestModel) -> Result<()> {
        // Delete associated reviews
        diesel::delete(review::table.filter(review::pull_request_id.eq(entry.id)))
            .execute_async(&self.pool)
            .await?;

        diesel::delete(pull_request::table.filter(pull_request::id.eq(entry.id)))
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }

    async fn remove_closed_pulls_from_repository(&self, repository_id: i32) -> Result<()> {
        diesel::delete(
            pull_request::table
                .filter(pull_request::repository_id.eq(repository_id))
                .filter(pull_request::closed.eq(true)),
        )
        .execute_async(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, entry: &mut PullRequestModel) -> Result<()> {
        let copy = entry.clone();

        *entry = diesel::update(pull_request::table.filter(pull_request::id.eq(copy.id)))
            .set(copy)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy pull request DB adapter.
pub struct DummyPullRequestDbAdapter {
    /// Create response.
    pub create_response: Mock<Option<Result<PullRequestModel>>>,
    /// Fetch status comment ID response.
    pub fetch_status_comment_id_response: Mock<Result<i32>>,
    /// List response.
    pub list_response: Mock<Result<Vec<PullRequestModel>>>,
    /// List from repository path response.
    pub list_from_repository_path_response: Mock<Result<Vec<PullRequestModel>>>,
    /// Get from repository and number response.
    pub get_from_repository_and_number_response: Mock<Result<PullRequestModel>>,
    /// Get from repository path and number response.
    pub get_from_repository_path_and_number_response:
        Mock<Result<(PullRequestModel, RepositoryModel)>>,
    /// List closed pulls from repository response.
    pub list_closed_pulls_from_repository_response: Mock<Result<Vec<PullRequestModel>>>,
    /// Remove response.
    pub remove_response: Mock<Result<()>>,
    /// Remove closed pulls from repository response.
    pub remove_closed_pulls_from_repository_response: Mock<Result<()>>,
    /// Save response.
    pub save_response: Mock<Result<()>>,
}

impl Default for DummyPullRequestDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(None),
            fetch_status_comment_id_response: Mock::new(Ok(0)),
            list_response: Mock::new(Ok(Vec::new())),
            list_from_repository_path_response: Mock::new(Ok(Vec::new())),
            get_from_repository_and_number_response: Mock::new(Ok(PullRequestModel::default())),
            get_from_repository_path_and_number_response: Mock::new(Ok((
                PullRequestModel::default(),
                RepositoryModel::default(),
            ))),
            list_closed_pulls_from_repository_response: Mock::new(Ok(Vec::new())),
            remove_response: Mock::new(Ok(())),
            remove_closed_pulls_from_repository_response: Mock::new(Ok(())),
            save_response: Mock::new(Ok(())),
        }
    }
}

impl DummyPullRequestDbAdapter {
    /// Creates a new dummy pull request DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IPullRequestDbAdapter for DummyPullRequestDbAdapter {
    async fn create(&self, entry: PullRequestCreation) -> Result<PullRequestModel> {
        self.create_response
            .response()
            .map_or_else(|| Ok(entry.into()), |r| r)
    }

    async fn fetch_status_comment_id(&self, pull_request_id: i32) -> Result<i32> {
        self.fetch_status_comment_id_response.response()
    }

    async fn list(&self) -> Result<Vec<PullRequestModel>> {
        self.list_response.response()
    }

    async fn list_from_repository_path(&self, path: &str) -> Result<Vec<PullRequestModel>> {
        self.list_from_repository_path_response.response()
    }

    async fn get_from_repository_and_number(
        &self,
        repository: &RepositoryModel,
        number: u64,
    ) -> Result<PullRequestModel> {
        self.get_from_repository_and_number_response.response()
    }

    async fn get_from_repository_path_and_number(
        &self,
        path: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        self.get_from_repository_path_and_number_response.response()
    }

    async fn list_closed_pulls_from_repository(
        &self,
        repository_id: i32,
    ) -> Result<Vec<PullRequestModel>> {
        self.list_closed_pulls_from_repository_response.response()
    }

    async fn remove(&self, entry: &PullRequestModel) -> Result<()> {
        self.remove_response.response()
    }

    async fn remove_closed_pulls_from_repository(&self, repository_id: i32) -> Result<()> {
        self.remove_closed_pulls_from_repository_response.response()
    }

    async fn save(&self, entry: &mut PullRequestModel) -> Result<()> {
        self.save_response.response()
    }
}
