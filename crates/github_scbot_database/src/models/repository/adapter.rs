use async_trait::async_trait;
use diesel::prelude::*;
use github_scbot_utils::Mock;
use tokio_diesel::AsyncRunQueryDsl;

use super::{RepositoryCreation, RepositoryModel, RepositoryUpdate};
use crate::{schema::repository, DatabaseError, DbPool, Result};

/// Repository DB adapter.
#[async_trait]
pub trait IRepositoryDbAdapter {
    /// Creates a new repository.
    async fn create(&self, entry: RepositoryCreation) -> Result<RepositoryModel>;
    /// Lists available repositories.
    async fn list(&self) -> Result<Vec<RepositoryModel>>;
    /// Gets repository from ID.
    async fn get_from_id(&self, id: i32) -> Result<RepositoryModel>;
    /// Gets repository from owner and name.
    async fn get_from_owner_and_name(&self, owner: &str, name: &str) -> Result<RepositoryModel>;
    /// Updates repository.
    async fn update(&self, entry: &mut RepositoryModel, update: RepositoryUpdate) -> Result<()>;
}

/// Concrete repository DB adapter.
pub struct RepositoryDbAdapter {
    pool: DbPool,
}

impl RepositoryDbAdapter {
    /// Creates a new repository DB adapter.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IRepositoryDbAdapter for RepositoryDbAdapter {
    async fn create(&self, entry: RepositoryCreation) -> Result<RepositoryModel> {
        diesel::insert_into(repository::table)
            .values(entry)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list(&self) -> Result<Vec<RepositoryModel>> {
        repository::table
            .load_async::<RepositoryModel>(&self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn get_from_id(&self, id: i32) -> Result<RepositoryModel> {
        repository::table
            .filter(repository::id.eq(id))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownRepository(format!("<ID {}>", id)))
    }

    async fn get_from_owner_and_name(&self, owner: &str, name: &str) -> Result<RepositoryModel> {
        let owner = owner.to_owned();
        let name = name.to_owned();

        repository::table
            .filter(repository::name.eq(name.clone()))
            .filter(repository::owner.eq(owner.clone()))
            .first_async(&self.pool)
            .await
            .map_err(|_e| DatabaseError::UnknownRepository(format!("{0}/{1}", owner, name)))
    }

    async fn update(&self, entry: &mut RepositoryModel, update: RepositoryUpdate) -> Result<()> {
        *entry = diesel::update(repository::table.filter(repository::id.eq(entry.id)))
            .set(update)
            .get_result_async(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy repository DB adapter.
pub struct DummyRepositoryDbAdapter {
    /// Create response.
    pub create_response: Mock<RepositoryCreation, Result<RepositoryModel>>,
    /// List response.
    pub list_response: Mock<(), Result<Vec<RepositoryModel>>>,
    /// Get from ID response.
    pub get_from_id_response: Mock<i32, Result<RepositoryModel>>,
    /// Get from owner and name response.
    pub get_from_owner_and_name_response: Mock<(String, String), Result<RepositoryModel>>,
}

impl Default for DummyRepositoryDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(Box::new(|e| Ok(e.into()))),
            list_response: Mock::new(Box::new(|_| Ok(Vec::new()))),
            get_from_id_response: Mock::new(Box::new(|_| Ok(RepositoryModel::default()))),
            get_from_owner_and_name_response: Mock::new(Box::new(|_| {
                Ok(RepositoryModel::default())
            })),
        }
    }
}

impl DummyRepositoryDbAdapter {
    /// Creates a new dummy repository DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IRepositoryDbAdapter for DummyRepositoryDbAdapter {
    async fn create(&self, entry: RepositoryCreation) -> Result<RepositoryModel> {
        self.create_response.call(entry)
    }

    async fn list(&self) -> Result<Vec<RepositoryModel>> {
        self.list_response.call(())
    }

    async fn get_from_id(&self, id: i32) -> Result<RepositoryModel> {
        self.get_from_id_response.call(id)
    }

    async fn get_from_owner_and_name(&self, owner: &str, name: &str) -> Result<RepositoryModel> {
        self.get_from_owner_and_name_response
            .call((owner.to_owned(), name.to_owned()))
    }

    async fn update(&self, entry: &mut RepositoryModel, update: RepositoryUpdate) -> Result<()> {
        entry.apply_local_update(update);
        Ok(())
    }
}
