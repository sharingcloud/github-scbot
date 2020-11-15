//! SC Bot entrypoint

use dotenv::dotenv;
use github_scbot::run_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    run_server("127.0.0.1:8008").await
}
