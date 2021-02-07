//! Database module.

#![warn(missing_docs)]
#![warn(clippy::all)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::prelude::*;
use github_scbot_core::Config;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

pub mod errors;
pub mod import_export;
pub mod models;
mod schema;

#[cfg(test)]
mod tests;

pub use errors::{DatabaseError, Result};

/// Database connection alias.
pub type DbConn = PgConnection;
/// Database pool alias.
pub type DbPool = Pool<ConnectionManager<DbConn>>;

embed_migrations!();

/// Establish a single database connection.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn establish_single_connection(config: &Config) -> Result<DbConn> {
    ConnectionBuilder::configure(config).build()
}

/// Establish a single test database connection.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn establish_single_test_connection(config: &Config) -> Result<DbConn> {
    ConnectionBuilder::configure_for_test(config).build()
}

/// Establish a connection to a database pool.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn establish_connection(config: &Config) -> Result<DbPool> {
    ConnectionBuilder::configure(config).build_pool()
}

/// Establish a connection to a test database pool.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn establish_test_connection(config: &Config) -> Result<DbPool> {
    ConnectionBuilder::configure_for_test(config).build_pool()
}

struct ConnectionBuilder {
    database_url: String,
    test_mode: bool,
}

impl ConnectionBuilder {
    fn configure(config: &Config) -> Self {
        Self {
            database_url: config.database_url.clone(),
            test_mode: false,
        }
    }

    fn configure_for_test(config: &Config) -> Self {
        Self {
            database_url: config.test_database_url.clone(),
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
