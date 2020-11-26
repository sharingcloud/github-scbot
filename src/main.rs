//! SC Bot entrypoint

use color_eyre::eyre::Result;
use dotenv::dotenv;
use github_scbot::run_server;

use actix_web::rt;

fn main() -> Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    run_bot_server()?;

    Ok(())
}

fn run_bot_server() -> Result<(), std::io::Error> {
    let mut sys = rt::System::new("app");
    sys.block_on(run_server("127.0.0.1:8008"))
}
