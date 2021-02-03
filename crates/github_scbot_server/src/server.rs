/// Server module.
use actix_web::{middleware::Logger, rt, web, App, HttpServer};
use github_scbot_core::constants::{ENV_BIND_IP, ENV_BIND_PORT};
use github_scbot_database::establish_connection;
use sentry_actix::Sentry;
use tracing::{error, info};

use crate::{configure_webhooks, sentry_utils::with_sentry_configuration, Result, VerifySignature};

/// Run bot server.
pub fn run_bot_server() -> Result<()> {
    // Intro message
    info!("Starting bot server v{} ...", env!("CARGO_PKG_VERSION"));

    with_sentry_configuration(|| {
        let mut sys = rt::System::new("app");
        sys.block_on(run_bot_server_internal(get_bind_address()))
    })
}

fn get_bind_address() -> String {
    let ip = std::env::var(ENV_BIND_IP).unwrap();
    let port = std::env::var(ENV_BIND_PORT).unwrap();
    format!("{}:{}", ip, port)
}

async fn run_bot_server_internal(ip_with_port: String) -> Result<()> {
    if let Ok(pool) = establish_connection() {
        HttpServer::new(move || {
            App::new()
                .data(pool.clone())
                .wrap(Sentry::new())
                .wrap(VerifySignature::new())
                .wrap(Logger::default())
                .service(web::scope("/webhook").configure(configure_webhooks))
                .route("/", web::get().to(|| async { "Welcome on SC Bot!" }))
        })
        .bind(ip_with_port)?
        .run()
        .await
        .map_err(Into::into)
    } else {
        error!("Error while establishing connection to database.");
        std::process::exit(1);
    }
}
