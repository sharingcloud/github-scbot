mod errors;
pub(crate) mod models;
pub(crate) mod utils;

use std::ops::Deref;

pub use models::account::{Account, AccountDB, AccountDBImpl, AccountDBImplPool};
pub use models::external_account::{
    ExternalAccount, ExternalAccountDB, ExternalAccountDBImpl, ExternalAccountDBImplPool,
};
pub use models::repository::{Repository, RepositoryDB, RepositoryDBImpl, RepositoryDBImplPool};

pub use errors::{DatabaseError, Result};
pub type DbPool = sqlx::postgres::PgPool;

use github_scbot_conf::Config;
use sqlx::{migrate::Migrate, postgres::PgPoolOptions, Acquire};

pub async fn run_migrations<'a, A>(migrator: A) -> Result<()>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations")
        .run(migrator)
        .await
        .map_err(|_| DatabaseError::MigrationError)?;

    Ok(())
}

pub async fn establish_pool_connection(config: Config) -> Result<DbPool> {
    PgPoolOptions::new()
        .max_connections(config.database_pool_size)
        .connect(&config.database_url)
        .await
        .map_err(DatabaseError::ConnectionError)
}

#[cfg(test)]
mod tests {
    use crate::errors::StdError;

    use super::*;

    #[actix_rt::test]
    async fn test_init() -> core::result::Result<(), StdError> {
        let config = Config::from_env();

        let conn = establish_pool_connection(config).await?;
        let mut transaction = conn.begin().await?;
        run_migrations(&mut transaction).await?;

        // Ok, you can use transaction now

        Ok(())
    }
}
