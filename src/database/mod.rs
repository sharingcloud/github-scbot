//! Database module

use std::env;

use diesel::prelude::*;
use eyre::{eyre, Result, WrapErr};
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

mod constants;
pub mod models;
pub mod schema;

#[cfg(test)]
mod tests;

use constants::{ENV_DATABASE_URL, ENV_TEST_DATABASE_URL};

embed_migrations!();

pub type DbConn = PgConnection;
pub type DbPool = Pool<ConnectionManager<DbConn>>;

pub fn run_migrations(conn: &PgConnection) -> Result<()> {
    diesel_migrations::run_pending_migrations(conn).map_err(Into::into)
}

pub fn establish_connection() -> Result<DbPool> {
    if cfg!(test) {
        let test_database_url = env::var(ENV_TEST_DATABASE_URL)
            .wrap_err_with(|| (format!("{} must be set", ENV_TEST_DATABASE_URL)))?;
        let manager = ConnectionManager::<PgConnection>::new(&test_database_url);
        let pool = Pool::builder().build(manager)?;

        if let Ok(conn) = pool.get() {
            conn.begin_test_transaction()?;
            run_migrations(&conn)?;
        } else {
            return Err(eyre!("Error while establishing connection to database."));
        }

        Ok(pool)
    } else {
        let database_url = env::var(ENV_DATABASE_URL)
            .wrap_err_with(|| (format!("{} must be set", ENV_DATABASE_URL)))?;
        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        Pool::builder().build(manager).map_err(Into::into)
    }
}
