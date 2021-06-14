use serde::{Deserialize, Serialize};

use crate::schema::account;

mod adapter;
mod builder;
pub use adapter::{AccountDbAdapter, DummyAccountDbAdapter, IAccountDbAdapter};
use builder::AccountModelBuilder;

/// Account model.
#[derive(
    Debug,
    Deserialize,
    Insertable,
    Identifiable,
    Serialize,
    Queryable,
    Clone,
    AsChangeset,
    Default,
    PartialEq,
    Eq,
)]
#[primary_key(username)]
#[table_name = "account"]
pub struct AccountModel {
    /// Username.
    pub username: String,
    /// Is admin?
    pub is_admin: bool,
}

impl AccountModel {
    /// Create builder.
    pub fn builder<T: Into<String>>(username: T) -> AccountModelBuilder {
        AccountModelBuilder::default(username)
    }

    /// Create builder from model.
    pub fn builder_from_model(model: &Self) -> AccountModelBuilder {
        AccountModelBuilder::from_model(model)
    }
}
