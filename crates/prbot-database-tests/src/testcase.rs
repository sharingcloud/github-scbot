use futures::Future;
use prbot_config::Config;
use prbot_database_interface::DbService;
use prbot_database_memory::MemoryDb;
use prbot_database_pg::{
    create_db_pool_connection, create_db_url, get_base_url, setup_test_db, teardown_test_db,
    PostgresDb,
};
use tracing::info;

pub async fn db_test_case<F, Fut>(test_name: &str, block: F)
where
    F: Fn(Box<dyn DbService>) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>,
{
    let mut config = Config::from_env_no_version();
    let full_name = format!("test-bot-{test_name}");
    let base_url = get_base_url(&config.database.pg.url);
    let new_url = create_db_url(&base_url, &full_name);
    config.database.pg.url.clone_from(&new_url);
    config.database.pg.pool_size = 2;
    config.database.pg.connection_timeout = 5;

    {
        // In memory
        let mem_db = Box::new(MemoryDb::new());
        info!("running memory test {full_name} ...");
        block(mem_db).await.unwrap();
    }

    {
        // Postgres
        setup_test_db(&base_url, &full_name).await;

        let pool = create_db_pool_connection(&config, &base_url, &full_name).await;
        let pg_db = Box::new(PostgresDb::new(pool));
        info!("running postgres test {full_name} ...");
        block(pg_db).await.unwrap();

        teardown_test_db(&base_url, &full_name).await;
    }
}

pub async fn db_test_case_pg<F, Fut>(test_name: &str, block: F)
where
    F: Fn(Box<dyn DbService>) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>,
{
    let mut config = Config::from_env_no_version();
    let full_name = format!("test-bot-{test_name}");
    let base_url = get_base_url(&config.database.pg.url);
    let new_url = create_db_url(&base_url, &full_name);
    config.database.pg.url.clone_from(&new_url);
    config.database.pg.pool_size = 2;
    config.database.pg.connection_timeout = 5;

    {
        // Postgres
        setup_test_db(&base_url, &full_name).await;

        let pool = create_db_pool_connection(&config, &base_url, &full_name).await;
        let pg_db = Box::new(PostgresDb::new(pool));
        info!("running postgres test {full_name} ...");
        block(pg_db).await.unwrap();

        teardown_test_db(&base_url, &full_name).await;
    }
}
