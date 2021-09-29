use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
use tokio_diesel::AsyncRunQueryDsl;

use crate::{
    models::{ExternalAccountRightModel, RepositoryModel},
    schema::external_account_right,
    DatabaseError, DbPool, Result,
};

/// External account right DB adapter.
#[async_trait]
pub trait IExternalAccountRightDbAdapter {
    /// Lists available external account rights.
    async fn list(&self) -> Result<Vec<ExternalAccountRightModel>>;
    /// Lists available external accounts rights for username.
    async fn list_rights(&self, username: &str) -> Result<Vec<ExternalAccountRightModel>>;
    /// Gets external account right for username on repository.
    async fn get_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel>;
    /// Adds right to username on repository.
    async fn add_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel>;
    /// Removes right from username on repository.
    async fn remove_right(&self, username: &str, repository: &RepositoryModel) -> Result<()>;
    /// Removes all rights from username.
    async fn remove_rights(&self, username: &str) -> Result<()>;
}

/// Concrete external account right DB adapter.
pub struct ExternalAccountRightDbAdapter {
    pool: DbPool,
}

impl ExternalAccountRightDbAdapter {
    /// Creates a new external account right DB adapter.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IExternalAccountRightDbAdapter for ExternalAccountRightDbAdapter {
    async fn list(&self) -> Result<Vec<ExternalAccountRightModel>> {
        external_account_right::table
            .load_async::<ExternalAccountRightModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list_rights(&self, username: &str) -> Result<Vec<ExternalAccountRightModel>> {
        let username = username.to_owned();

        external_account_right::table
            .filter(external_account_right::username.eq(username))
            .get_results_async(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn get_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel> {
        let username = username.to_owned();
        let repository = repository.clone();

        external_account_right::table
            .filter(external_account_right::username.eq(username.clone()))
            .filter(external_account_right::repository_id.eq(repository.id))
            .first_async(&self.pool)
            .await
            .map_err(|_e| {
                DatabaseError::UnknownExternalAccountRight(username, repository.get_path())
            })
    }

    async fn add_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel> {
        if let Ok(existing) = self.get_right(username, repository).await {
            Ok(existing)
        } else {
            let entry = ExternalAccountRightModel {
                username: username.into(),
                repository_id: repository.id,
            };

            Ok(diesel::insert_into(external_account_right::table)
                .values(entry)
                .get_result_async(&self.pool)
                .await
                .map_err(DatabaseError::from)?)
        }
    }

    async fn remove_right(&self, username: &str, repository: &RepositoryModel) -> Result<()> {
        let username = username.to_owned();
        let repository = repository.clone();

        diesel::delete(
            external_account_right::table
                .filter(external_account_right::username.eq(username))
                .filter(external_account_right::repository_id.eq(repository.id)),
        )
        .execute_async(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_rights(&self, username: &str) -> Result<()> {
        let username = username.to_owned();

        diesel::delete(
            external_account_right::table.filter(external_account_right::username.eq(username)),
        )
        .execute_async(&self.pool)
        .await?;

        Ok(())
    }
}

/// Dummy external account right DB adapter.
pub struct DummyExternalAccountRightDbAdapter {
    /// List response.
    pub list_response: Mock<Result<Vec<ExternalAccountRightModel>>>,
    /// Get repository response.
    pub get_repository_response: Mock<Result<RepositoryModel>>,
    /// List rights response.
    pub list_rights_response: Mock<Result<Vec<ExternalAccountRightModel>>>,
    /// Get right response.
    pub get_right_response: Mock<Result<ExternalAccountRightModel>>,
    /// Add right response.
    pub add_right_response: Mock<Result<ExternalAccountRightModel>>,
    /// Remove right response.
    pub remove_right_response: Mock<Result<()>>,
    /// Remove rights response.
    pub remove_rights_response: Mock<Result<()>>,
}

impl Default for DummyExternalAccountRightDbAdapter {
    fn default() -> Self {
        Self {
            list_response: Mock::new(Ok(Vec::new())),
            get_repository_response: Mock::new(Ok(RepositoryModel::default())),
            list_rights_response: Mock::new(Ok(Vec::new())),
            get_right_response: Mock::new(Ok(ExternalAccountRightModel::default())),
            add_right_response: Mock::new(Ok(ExternalAccountRightModel::default())),
            remove_right_response: Mock::new(Ok(())),
            remove_rights_response: Mock::new(Ok(())),
        }
    }
}

impl DummyExternalAccountRightDbAdapter {
    /// Creates a new dummy external account right DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IExternalAccountRightDbAdapter for DummyExternalAccountRightDbAdapter {
    async fn list(&self) -> Result<Vec<ExternalAccountRightModel>> {
        self.list_response.response()
    }

    async fn list_rights(&self, username: &str) -> Result<Vec<ExternalAccountRightModel>> {
        self.list_rights_response.response()
    }

    async fn get_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel> {
        self.get_right_response.response()
    }

    async fn add_right(
        &self,
        username: &str,
        repository: &RepositoryModel,
    ) -> Result<ExternalAccountRightModel> {
        self.add_right_response.response()
    }

    async fn remove_right(&self, username: &str, repository: &RepositoryModel) -> Result<()> {
        self.remove_right_response.response()
    }

    async fn remove_rights(&self, username: &str) -> Result<()> {
        self.remove_rights_response.response()
    }
}
