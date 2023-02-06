mod errors;
pub(crate) mod exchange;
pub(crate) mod fields;
pub(crate) mod interface;
pub(crate) mod memory;
pub(crate) mod models;
pub(crate) mod postgres;
pub(crate) mod utils;

use std::ops::Deref;
use std::time::Duration;

pub use exchange::Exchanger;
pub use models::account::Account;
pub use models::external_account::{ExternalAccount, ExternalJwtClaims};
pub use models::external_account_right::ExternalAccountRight;
pub use models::merge_rule::MergeRule;
pub use models::pull_request::PullRequest;
pub use models::repository::Repository;
pub use models::required_reviewer::RequiredReviewer;

pub use errors::{DatabaseError, Result};
pub use interface::DbServiceAll;
pub use memory::MemoryDb;
pub use postgres::PostgresDb;
pub use utils::db_test_case;
pub type DbPool = sqlx::postgres::PgPool;

use github_scbot_core::config::Config;
use sqlx::{migrate::Migrate, postgres::PgPoolOptions, Acquire};

pub async fn run_migrations<'a, A>(migrator: A) -> Result<()>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations")
        .run(migrator)
        .await
        .map_err(|e| DatabaseError::MigrationError { source: e })?;

    Ok(())
}

pub async fn establish_pool_connection(config: &Config) -> Result<DbPool> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(
            config.database_connection_timeout.into(),
        ))
        .max_connections(config.database_pool_size)
        .connect(&config.database_url)
        .await
        .map_err(|e| DatabaseError::ConnectionError { source: e })
}
