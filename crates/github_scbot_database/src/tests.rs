//! Test utils

use std::{future::Future, sync::Arc};

use diesel::{r2d2::ConnectionManager, Connection, PgConnection, RunQueryDsl};
use github_scbot_conf::Config;
use r2d2::Pool;

use crate::{DbPool, Result};

fn get_base_url(config: &Config) -> String {
    config
        .test_database_url
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/")
}

fn create_postgres_connection(base_url: &str) -> Result<PgConnection> {
    let url = format!("{}/postgres", base_url);
    Ok(PgConnection::establish(&url)?)
}

fn create_db_connection(base_url: &str, db_name: &str) -> Result<PgConnection> {
    let url = format!("{}/{}", base_url, db_name);
    Ok(PgConnection::establish(&url)?)
}

fn create_pool(base_url: &str, db_name: &str) -> Result<DbPool> {
    let url = format!("{}/{}", base_url, db_name);
    let manager = ConnectionManager::<PgConnection>::new(&url);
    Ok(Pool::builder().build(manager)?)
}

fn terminate_connections(conn: &PgConnection, db_name: &str) -> Result<()> {
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

fn create_database(conn: &PgConnection, db_name: &str) -> Result<()> {
    diesel::sql_query(format!(r#"CREATE DATABASE {};"#, db_name)).execute(conn)?;

    Ok(())
}

fn drop_database(conn: &PgConnection, db_name: &str) -> Result<()> {
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
#[allow(clippy::missing_panics_doc)]
pub async fn using_test_db<F, Fut, E>(db_name: &str, test: F) -> Result<()>
where
    E: std::fmt::Debug,
    F: FnOnce(Config, Arc<DbPool>) -> Fut,
    Fut: Future<Output = core::result::Result<(), E>>,
{
    let mut config = Config::from_env();
    config.bot_username = "test-bot".into();

    let base_url = get_base_url(&config);
    teardown_test_db(&base_url, db_name)?;
    setup_test_db(&base_url, db_name)?;

    let pool = Arc::new(create_pool(&base_url, db_name)?);
    let result = test(config, pool).await;
    teardown_test_db(&base_url, db_name)?;

    if let Err(e) = result {
        panic!("Error while executing test: {:?}", e);
    }

    Ok(())
}
