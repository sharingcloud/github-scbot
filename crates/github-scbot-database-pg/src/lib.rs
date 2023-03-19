mod fields;
mod postgres;
mod row;
mod utils;

use std::{ops::Deref, time::Duration};

use github_scbot_config::Config;
use github_scbot_database_interface::{DatabaseError, Result};
use sqlx::{migrate::Migrate, postgres::PgPoolOptions, Acquire};

pub type DbPool = sqlx::postgres::PgPool;
pub use postgres::PostgresDb;
pub use utils::{
    create_db_pool_connection, create_db_url, get_base_url, setup_test_db, teardown_test_db,
};

pub async fn run_migrations<'a, A>(migrator: A) -> Result<()>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations")
        .run(migrator)
        .await
        .map_err(|e| DatabaseError::ImplementationError { source: e.into() })?;

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
        .map_err(|e| DatabaseError::ImplementationError { source: e.into() })
}
