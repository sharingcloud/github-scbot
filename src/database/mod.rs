//! Database module

use std::env;

use diesel::prelude::*;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

pub mod constants;
pub mod models;
pub mod schema;

#[cfg(test)]
mod tests;

use crate::errors::{BotError, Result};
use constants::{ENV_DATABASE_URL, ENV_TEST_DATABASE_URL};

embed_migrations!();

pub type DbConn = PgConnection;
pub type DbPool = Pool<ConnectionManager<DbConn>>;

pub fn establish_single_connection() -> Result<DbConn> {
    ConnectionBuilder::configure()?.build()
}

pub fn establish_connection() -> Result<DbPool> {
    ConnectionBuilder::configure()?.build_pool()
}

struct ConnectionBuilder {
    database_url: String,
    test_mode: bool,
}

impl ConnectionBuilder {
    fn configure() -> Result<Self> {
        if cfg!(test) {
            Self::configure_for_test()
        } else {
            Ok(Self {
                database_url: env::var(ENV_DATABASE_URL).map_err(|_e| {
                    BotError::ConfigurationError(format!("{} must be set", ENV_DATABASE_URL))
                })?,
                test_mode: false,
            })
        }
    }

    fn configure_for_test() -> Result<Self> {
        Ok(Self {
            database_url: env::var(ENV_TEST_DATABASE_URL).map_err(|_e| {
                BotError::ConfigurationError(format!("{} must be set", ENV_TEST_DATABASE_URL))
            })?,
            test_mode: true,
        })
    }

    fn build(self) -> Result<DbConn> {
        let conn = PgConnection::establish(&self.database_url)
            .map_err(|e| BotError::DBError(e.to_string()))?;

        if self.test_mode {
            Self::prepare_connection_for_testing(&conn)?;
        }

        Ok(conn)
    }

    fn build_pool(self) -> Result<DbPool> {
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        let pool = Pool::builder()
            .build(manager)
            .map_err(|e| BotError::DBError(e.to_string()))?;
        let conn = pool.get().map_err(|e| BotError::DBError(e.to_string()))?;

        if self.test_mode {
            Self::prepare_connection_for_testing(&conn)?;
        }

        Ok(pool)
    }

    fn prepare_connection_for_testing(conn: &DbConn) -> Result<()> {
        conn.begin_test_transaction()
            .map_err(|e| BotError::DBError(e.to_string()))?;
        diesel_migrations::run_pending_migrations(conn)
            .map_err(|e| BotError::DBError(e.to_string()))?;

        Ok(())
    }
}
