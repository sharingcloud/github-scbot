use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
use tokio_diesel::AsyncRunQueryDsl;

use super::{PullRequestCreation, PullRequestModel, PullRequestUpdate};
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
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)>;
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
    /// Set status comment ID.
    async fn set_status_comment_id(
        &self,
        repo_owner: &str,
        repo_name: &str,
        issue_number: u64,
        comment_id: u64,
    ) -> Result<()>;
    /// Removes an existing pull request.
    async fn remove(&self, entry: &PullRequestModel) -> Result<()>;
    /// Removes closed pull requests from a repository.
    async fn remove_closed_pulls_from_repository(&self, repository_id: i32) -> Result<()>;
    /// Update.
    async fn update(&self, entry: &mut PullRequestModel, update: PullRequestUpdate) -> Result<()>;
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
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        let owner = owner.to_owned();
        let name = name.to_owned();
        let path = format!("{owner}/{name}");

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

    async fn get_from_repository_path_and_number(
        &self,
        path: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;
        self.get_from_repository_and_number(owner, name, number)
            .await
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

    async fn set_status_comment_id(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        comment_id: u64,
    ) -> Result<()> {
        let (pr, _) = self
            .get_from_repository_and_number(owner, name, number)
            .await?;

        diesel::update(pull_request::table.filter(pull_request::id.eq(pr.id())))
            .set(pull_request::status_comment_id.eq(comment_id as i32))
            .execute_async(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, entry: &mut PullRequestModel, update: PullRequestUpdate) -> Result<()> {
        *entry = diesel::update(pull_request::table.filter(pull_request::id.eq(entry.id)))
            .set(update)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy pull request DB adapter.
pub struct DummyPullRequestDbAdapter {
    /// Create response.
    pub create_response: Mock<PullRequestCreation, Result<PullRequestModel>>,
    /// Fetch status comment ID response.
    pub fetch_status_comment_id_response: Mock<i32, Result<i32>>,
    /// List response.
    pub list_response: Mock<(), Result<Vec<PullRequestModel>>>,
    /// List from repository path response.
    pub list_from_repository_path_response: Mock<String, Result<Vec<PullRequestModel>>>,
    /// Get from repository and number response.
    pub get_from_repository_and_number_response:
        Mock<(String, String, u64), Result<(PullRequestModel, RepositoryModel)>>,
    /// Get from repository path and number response.
    pub get_from_repository_path_and_number_response:
        Mock<(String, u64), Result<(PullRequestModel, RepositoryModel)>>,
    /// List closed pulls from repository response.
    pub list_closed_pulls_from_repository_response: Mock<i32, Result<Vec<PullRequestModel>>>,
    /// Set status comment ID.
    pub set_status_comment_id_response: Mock<(String, String, u64, u64), Result<()>>,
    /// Remove response.
    pub remove_response: Mock<PullRequestModel, Result<()>>,
    /// Remove closed pulls from repository response.
    pub remove_closed_pulls_from_repository_response: Mock<i32, Result<()>>,
}

impl Default for DummyPullRequestDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(Box::new(|e| Ok(e.into()))),
            fetch_status_comment_id_response: Mock::new(Box::new(|_| Ok(0))),
            list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            list_from_repository_path_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            get_from_repository_and_number_response: Mock::new(Box::new(|_| {
                Ok((PullRequestModel::default(), RepositoryModel::default()))
            })),
            get_from_repository_path_and_number_response: Mock::new(Box::new(|_| {
                Ok((PullRequestModel::default(), RepositoryModel::default()))
            })),
            list_closed_pulls_from_repository_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            set_status_comment_id_response: Mock::new(Box::new(|_| Ok(()))),
            remove_response: Mock::new(Box::new(|_| Ok(()))),
            remove_closed_pulls_from_repository_response: Mock::new(Box::new(|_| Ok(()))),
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
        self.create_response.call(entry)
    }

    async fn fetch_status_comment_id(&self, pull_request_id: i32) -> Result<i32> {
        self.fetch_status_comment_id_response.call(pull_request_id)
    }

    async fn list(&self) -> Result<Vec<PullRequestModel>> {
        self.list_response.call(())
    }

    async fn list_from_repository_path(&self, path: &str) -> Result<Vec<PullRequestModel>> {
        self.list_from_repository_path_response
            .call(path.to_owned())
    }

    async fn get_from_repository_and_number(
        &self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        self.get_from_repository_and_number_response.call((
            owner.to_owned(),
            name.to_owned(),
            number,
        ))
    }

    async fn get_from_repository_path_and_number(
        &self,
        path: &str,
        number: u64,
    ) -> Result<(PullRequestModel, RepositoryModel)> {
        self.get_from_repository_path_and_number_response
            .call((path.to_owned(), number))
    }

    async fn list_closed_pulls_from_repository(
        &self,
        repository_id: i32,
    ) -> Result<Vec<PullRequestModel>> {
        self.list_closed_pulls_from_repository_response
            .call(repository_id)
    }

    async fn set_status_comment_id(
        &self,
        owner: &str,
        name: &str,
        number: u64,
        comment_id: u64,
    ) -> Result<()> {
        self.set_status_comment_id_response.call((
            owner.to_owned(),
            name.to_owned(),
            number,
            comment_id,
        ))
    }

    async fn remove(&self, entry: &PullRequestModel) -> Result<()> {
        self.remove_response.call(entry.clone())
    }

    async fn remove_closed_pulls_from_repository(&self, repository_id: i32) -> Result<()> {
        self.remove_closed_pulls_from_repository_response
            .call(repository_id)
    }

    async fn update(&self, entry: &mut PullRequestModel, update: PullRequestUpdate) -> Result<()> {
        entry.apply_local_update(update);
        Ok(())
    }
}
