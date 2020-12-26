//! Server module

use actix_web::{middleware::Logger, rt, web, App, HttpServer};
use sentry_actix::Sentry;
use tracing::{error, info};

pub mod constants;

use crate::api::configure_debug_api;
use crate::database::establish_connection;
use crate::errors::Result;
use crate::utils::with_sentry_configuration;
use crate::webhook::{configure_webhooks, VerifySignature};

/// Run bot server.
///
/// # Errors
///
/// - `HttpServer` bind error
///
pub fn run_bot_server() -> Result<()> {
    // Intro message
    info!("Starting bot server v{} ...", env!("CARGO_PKG_VERSION"));

    with_sentry_configuration(|| {
        let mut sys = rt::System::new("app");
        sys.block_on(run_bot_server_internal(get_bind_address()))
    })
}

fn get_bind_address() -> String {
    let ip = std::env::var(constants::ENV_BIND_IP).unwrap();
    let port = std::env::var(constants::ENV_BIND_PORT).unwrap();
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
                .service(web::scope("/debug").configure(configure_debug_api))
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
