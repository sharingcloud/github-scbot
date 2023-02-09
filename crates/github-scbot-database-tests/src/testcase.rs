use futures::Future;
use github_scbot_core::config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_database_memory::MemoryDb;
use github_scbot_database_pg::{
    create_db_pool_connection, create_db_url, get_base_url, setup_test_db, teardown_test_db,
    PostgresDb,
};

pub async fn db_test_case<F, Fut>(test_name: &str, block: F)
where
    F: Fn(Box<dyn DbService>) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>,
{
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
