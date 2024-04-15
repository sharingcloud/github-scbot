mod fields;
mod postgres;
mod row;
mod utils;

use std::{ops::Deref, time::Duration};

use prbot_config::Config;
use prbot_database_interface::{DatabaseError, Result};
use sqlx::{migrate::Migrate, postgres::PgPoolOptions, Acquire};

pub type DbPool = sqlx::postgres::PgPool;
pub use postgres::PostgresDb;
use tracing::info;
pub use utils::{
    create_db_pool_connection, create_db_url, get_base_url, setup_test_db, teardown_test_db,
};

pub async fn run_migrations<'a, A>(migrator: A) -> Result<()>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(migrator)
        .await
        .map_err(|e| DatabaseError::ImplementationError { source: e.into() })?;

    Ok(())
}

pub async fn establish_pool_connection(config: &Config) -> Result<DbPool> {
    info!("Establishing connection to database pool...");

    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(
            config.database.pg.connection_timeout.into(),
        ))
        .max_connections(config.database.pg.pool_size)
        .connect(&config.database.pg.url)
        .await
        .map_err(|e| DatabaseError::ImplementationError { source: e.into() })
}
