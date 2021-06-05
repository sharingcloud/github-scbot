use serde::{Deserialize, Serialize};

use crate::{
    models::{IRepositoryDbAdapter, RepositoryModel},
    schema::external_account_right,
    Result,
};

mod adapter;
pub use adapter::{
    DummyExternalAccountRightDbAdapter, ExternalAccountRightDbAdapter,
    IExternalAccountRightDbAdapter,
};

/// External account right.
#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    Clone,
    PartialEq,
    Eq,
    Insertable,
    Default,
)]
#[primary_key(username, repository_id)]
#[table_name = "external_account_right"]
pub struct ExternalAccountRightModel {
    /// Username.
    pub username: String,
    /// Repository ID.
    pub repository_id: i32,
}

impl ExternalAccountRightModel {
    /// Gets repository.
    pub async fn get_repository(
        &self,
        repository_db_adapter: &dyn IRepositoryDbAdapter,
    ) -> Result<RepositoryModel> {
        repository_db_adapter.get_from_id(self.repository_id).await
    }
}
