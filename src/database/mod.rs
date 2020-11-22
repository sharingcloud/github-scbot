//! Database module

use std::env;

use diesel::sqlite::SqliteConnection;
use eyre::{eyre, Result, WrapErr};
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

mod constants;
pub mod models;
pub mod schema;

#[cfg(test)]
mod tests;

use constants::ENV_DATABASE_URL;

embed_migrations!();

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn run_migrations(conn: &SqliteConnection) -> Result<()> {
    diesel_migrations::run_pending_migrations(conn).map_err(Into::into)
}

pub fn establish_connection() -> Result<DbPool> {
    if cfg!(test) {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder().build(manager)?;

        if let Ok(conn) = pool.get() {
            run_migrations(&conn)?;
        } else {
            return Err(eyre!("Error while establising connection to database."));
        }

        Ok(pool)
    } else {
        let database_url = env::var(ENV_DATABASE_URL)
            .wrap_err_with(|| (format!("{} must be set", ENV_DATABASE_URL)))?;
        let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
        Pool::builder().build(manager).map_err(Into::into)
    }
}
