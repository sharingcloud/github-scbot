//! SC Bot entrypoint

use color_eyre::eyre::Result;
use dotenv::dotenv;
use github_scbot::run_server;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    color_eyre::install().expect("Error installing color-eyre");

    run_server("127.0.0.1:8008").await
}
