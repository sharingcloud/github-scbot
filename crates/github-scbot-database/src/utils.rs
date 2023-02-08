use futures::Future;
use github_scbot_core::config::Config;
use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};

use crate::{errors::StdError, run_migrations};

use crate::interface::DbService;

async fn create_postgres_connection(base_url: &str) -> PgConnection {
    create_connection(&create_db_url(base_url, "postgres")).await
}

async fn create_db_connection(base_url: &str, db_name: &str) -> PgConnection {
    create_connection(&create_db_url(base_url, db_name)).await
}

async fn create_db_pool_connection(config: &Config, base_url: &str, db_name: &str) -> PgPool {
    let db_url = create_db_url(base_url, db_name);

    PgPoolOptions::new()
        .max_connections(config.database_pool_size)
        .connect(&db_url)
        .await
        .unwrap()
}

async fn create_connection(db_url: &str) -> PgConnection {
    PgConnection::connect(db_url).await.unwrap()
}

async fn terminate_connections(conn: &mut PgConnection, db_name: &str) {
    sqlx::query(
        r#"
        SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE datname = $1
        AND pid <> pg_backend_pid();
    "#,
    )
    .bind(db_name)
    .execute(conn)
    .await
    .unwrap();
}

async fn create_database(conn: &mut PgConnection, db_name: &str) {
    sqlx::query(&format!(
        r#"
        CREATE DATABASE "{db_name}";
    "#
    ))
    .execute(conn)
    .await
    .unwrap();
}

async fn drop_database(conn: &mut PgConnection, db_name: &str) {
    sqlx::query(&format!(
        r#"
        DROP DATABASE IF EXISTS "{db_name}";
    "#
    ))
    .execute(conn)
    .await
    .unwrap();
}

fn get_base_url(url: &str) -> String {
    url.split('/').take(3).collect::<Vec<_>>().join("/")
}

async fn setup_test_db(base_url: &str, db_name: &str) {
    {
        let mut conn = create_postgres_connection(base_url).await;
        terminate_connections(&mut conn, db_name).await;
        drop_database(&mut conn, db_name).await;
        create_database(&mut conn, db_name).await;
    }

    {
        let mut conn = create_db_connection(base_url, db_name).await;
        run_migrations(&mut conn).await.unwrap();
    }
}

async fn teardown_test_db(base_url: &str, db_name: &str) {
    let mut conn = create_postgres_connection(base_url).await;
    terminate_connections(&mut conn, db_name).await;
    drop_database(&mut conn, db_name).await;
}

fn create_db_url(base_url: &str, db_name: &str) -> String {
    format!("{base_url}/{db_name}")
}

#[allow(unused)]
pub async fn db_test_case<F, Fut>(test_name: &str, block: F)
where
    F: Fn(Box<dyn DbService>) -> Fut,
    Fut: Future<Output = Result<(), StdError>>,
{
    use crate::{memory::MemoryDb, postgres::PostgresDb};

    let mut config = Config::from_env();
    let full_name = format!("test-bot-{test_name}");
    let base_url = get_base_url(&config.database_url);
    let new_url = create_db_url(&base_url, &full_name);
    config.database_url = new_url.clone();

    {
        // In memory
        let mem_db = Box::new(MemoryDb::new());
        println!("running memory test {full_name} ...");
        block(mem_db).await.unwrap();
    }

    {
        // Postgres
        teardown_test_db(&base_url, &full_name).await;
        setup_test_db(&base_url, &full_name).await;

        let pool = create_db_pool_connection(&config, &base_url, &full_name).await;
        let pg_db = Box::new(PostgresDb::new(pool));
        println!("running postgres test {full_name} ...");
        block(pg_db).await.unwrap();

        teardown_test_db(&base_url, &full_name).await;
    }
}
