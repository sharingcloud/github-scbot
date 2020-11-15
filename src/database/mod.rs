//! Database module

use std::env;

use diesel::sqlite::SqliteConnection;
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

pub fn run_migrations(conn: &SqliteConnection) {
    diesel_migrations::run_pending_migrations(conn).expect("Error while running migrations");
}

pub fn establish_connection() -> DbPool {
    if cfg!(test) {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create database pool.");
        run_migrations(&pool.get().unwrap());
        pool
    } else {
        let database_url = env::var(ENV_DATABASE_URL)
            .unwrap_or_else(|_| (panic!("{} must be set", ENV_DATABASE_URL)));
        let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
        Pool::builder()
            .build(manager)
            .unwrap_or_else(|_| panic!("Error connecting to database '{}'", database_url))
    }
}
