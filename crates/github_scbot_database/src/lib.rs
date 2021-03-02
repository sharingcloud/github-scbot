//! Database module.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Force include openssl for static linking
extern crate openssl;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::{prelude::*, r2d2::ConnectionManager};
use github_scbot_conf::Config;
use r2d2::{Pool, PooledConnection};

pub mod errors;
pub mod import_export;
pub mod models;
mod schema;

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

/// Establish a connection to a database pool.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn establish_pool_connection(config: &Config) -> Result<DbPool> {
    let pool = ConnectionBuilder::configure(config).build_pool()?;
    let conn = pool.get()?;

    // Apply migrations
    embedded_migrations::run(&*conn)?;

    Ok(pool)
}

/// Get connection from pool.
pub fn get_connection(pool: &DbPool) -> Result<PooledConnection<ConnectionManager<DbConn>>> {
    pool.get().map_err(Into::into)
}

struct ConnectionBuilder {
    database_url: String,
}

impl ConnectionBuilder {
    fn configure(config: &Config) -> Self {
        Self {
            database_url: config.database_url.clone(),
        }
    }

    fn build(self) -> Result<DbConn> {
        Ok(PgConnection::establish(&self.database_url)?)
    }

    fn build_pool(self) -> Result<DbPool> {
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        Ok(Pool::builder().build(manager)?)
    }
}

/// Test utils
pub mod tests {
    use std::future::Future;

    use diesel::{r2d2::ConnectionManager, Connection, PgConnection, RunQueryDsl};
    use github_scbot_conf::Config;
    use r2d2::Pool;

    use crate::{DbConn, DbPool, Result};

    fn get_base_url(config: &Config) -> String {
        config
            .test_database_url
            .split('/')
            .take(3)
            .collect::<Vec<_>>()
            .join("/")
    }

    fn create_postgres_connection(base_url: &str) -> Result<DbConn> {
        let url = format!("{}/postgres", base_url);
        Ok(PgConnection::establish(&url)?)
    }

    fn create_db_connection(base_url: &str, db_name: &str) -> Result<DbConn> {
        let url = format!("{}/{}", base_url, db_name);
        Ok(PgConnection::establish(&url)?)
    }

    fn create_pool(base_url: &str, db_name: &str) -> Result<DbPool> {
        let url = format!("{}/{}", base_url, db_name);
        let manager = ConnectionManager::<PgConnection>::new(&url);
        Ok(Pool::builder().build(manager)?)
    }

    fn terminate_connections(conn: &DbConn, db_name: &str) -> Result<()> {
        diesel::sql_query(format!(
            r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
               FROM pg_stat_activity
               WHERE datname = '{}'
               AND pid <> pg_backend_pid();"#,
            db_name
        ))
        .execute(conn)?;

        Ok(())
    }

    fn create_database(conn: &DbConn, db_name: &str) -> Result<()> {
        diesel::sql_query(format!(r#"CREATE DATABASE {};"#, db_name)).execute(conn)?;

        Ok(())
    }

    fn drop_database(conn: &DbConn, db_name: &str) -> Result<()> {
        diesel::sql_query(format!(r#"DROP DATABASE IF EXISTS {};"#, db_name)).execute(conn)?;

        Ok(())
    }

    fn setup_test_db(base_url: &str, db_name: &str) -> Result<()> {
        {
            let conn = create_postgres_connection(base_url)?;
            terminate_connections(&conn, db_name)?;
            drop_database(&conn, db_name)?;
            create_database(&conn, db_name)?;
        }

        {
            let conn = create_db_connection(base_url, db_name)?;
            diesel_migrations::run_pending_migrations(&conn)?;
        }

        Ok(())
    }

    fn teardown_test_db(base_url: &str, db_name: &str) -> Result<()> {
        let conn = create_postgres_connection(base_url)?;
        terminate_connections(&conn, db_name)?;
        drop_database(&conn, db_name)
    }

    /// Using test database.
    pub async fn using_test_db<F, Fut, E>(config: &Config, db_name: &str, test: F) -> Result<()>
    where
        E: std::fmt::Debug,
        F: FnOnce(DbPool) -> Fut,
        Fut: Future<Output = core::result::Result<(), E>>,
    {
        let base_url = get_base_url(config);
        setup_test_db(&base_url, db_name)?;
        {
            let pool = create_pool(&base_url, db_name)?;
            if let Err(e) = test(pool).await {
                teardown_test_db(&base_url, db_name)?;
                panic!("Error while executing test: {:?}", e);
            }
        }
        teardown_test_db(&base_url, db_name)
    }
}
