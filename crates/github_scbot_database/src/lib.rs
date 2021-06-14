//! Database module.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::{prelude::*, r2d2::ConnectionManager};
use github_scbot_conf::Config;
use r2d2::Pool;

mod errors;
pub mod import_export;
pub mod models;
mod schema;

pub use errors::{DatabaseError, Result};

/// Database pool alias.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub mod tests;

embed_migrations!();

/// Establish a connection to a database pool.
pub fn establish_pool_connection(config: &Config) -> Result<DbPool> {
    ConnectionBuilder::configure(config).build_pool()
}

/// Run migrations.
pub fn run_migrations(pool: &DbPool) -> Result<()> {
    embedded_migrations::run(&*pool.get()?)?;
    Ok(())
}

struct ConnectionBuilder {
    database_url: String,
    pool_size: u32,
}

impl ConnectionBuilder {
    fn configure(config: &Config) -> Self {
        Self {
            database_url: config.database_url.clone(),
            pool_size: config.database_pool_size,
        }
    }

    fn build_pool(self) -> Result<DbPool> {
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        Ok(Pool::builder().max_size(self.pool_size).build(manager)?)
    }
}
