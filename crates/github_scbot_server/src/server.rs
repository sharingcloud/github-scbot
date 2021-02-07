//! Server module.

use actix_web::{middleware::Logger, rt, web, App, HttpServer};
use github_scbot_core::Config;
use github_scbot_database::establish_connection;
use sentry_actix::Sentry;
use tracing::{error, info};

use crate::{configure_webhooks, sentry_utils::with_sentry_configuration, Result, VerifySignature};

/// Run bot server.
pub fn run_bot_server(config: Config) -> Result<()> {
    // Intro message
    info!("Starting bot server v{} ...", env!("CARGO_PKG_VERSION"));

    with_sentry_configuration(config.clone(), || {
        let mut sys = rt::System::new("app");
        let address = get_bind_address(&config);
        sys.block_on(run_bot_server_internal(config, address))
    })
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server_bind_ip, config.server_bind_port)
}

async fn run_bot_server_internal(config: Config, ip_with_port: String) -> Result<()> {
    if let Ok(pool) = establish_connection(&config) {
        HttpServer::new(move || {
            App::new()
                .data(config.clone())
                .data(pool.clone())
                .wrap(Sentry::new())
                .wrap(VerifySignature::new(&config.clone()))
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
