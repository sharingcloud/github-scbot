//! Database module.

use std::env;

use diesel::prelude::*;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

pub mod constants;
pub mod errors;
pub mod import_export;
pub mod models;
mod schema;

#[cfg(test)]
mod tests;

use constants::{ENV_DATABASE_URL, ENV_TEST_DATABASE_URL};
pub use errors::{DatabaseError, Result};

/// Database connection alias.
pub type DbConn = PgConnection;
/// Database pool alias.
pub type DbPool = Pool<ConnectionManager<DbConn>>;

embed_migrations!();

/// Establish a single database connection.
pub fn establish_single_connection() -> Result<DbConn> {
    ConnectionBuilder::configure().build()
}

/// Establish a connection to a database pool.
pub fn establish_connection() -> Result<DbPool> {
    ConnectionBuilder::configure().build_pool()
}

struct ConnectionBuilder {
    database_url: String,
    test_mode: bool,
}

impl ConnectionBuilder {
    fn configure() -> Self {
        if cfg!(test) {
            Self::configure_for_test()
        } else {
            Self {
                database_url: env::var(ENV_DATABASE_URL).unwrap(),
                test_mode: false,
            }
        }
    }

    fn configure_for_test() -> Self {
        Self {
            database_url: env::var(ENV_TEST_DATABASE_URL).unwrap(),
            test_mode: true,
        }
    }

    fn build(self) -> Result<DbConn> {
        let conn = PgConnection::establish(&self.database_url)?;

        if self.test_mode {
            Self::prepare_connection_for_testing(&conn)?;
        }

        Ok(conn)
    }

    fn build_pool(self) -> Result<DbPool> {
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        let pool = Pool::builder().build(manager)?;
        let conn = pool.get()?;

        if self.test_mode {
            Self::prepare_connection_for_testing(&conn)?;
        }

        Ok(pool)
    }

    fn prepare_connection_for_testing(conn: &DbConn) -> Result<()> {
        conn.begin_test_transaction()?;
        diesel_migrations::run_pending_migrations(conn)?;

        Ok(())
    }
}
