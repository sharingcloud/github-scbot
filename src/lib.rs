//! SC Bot library

#![deny(missing_docs)]

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

mod webhook;

/// Run bot server.
///
/// # Arguments
///
/// * `ip` - Bind IP address
///
pub async fn run_server(ip: &str) -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    HttpServer::new(|| {
        App::new()
            .wrap(webhook::VerifySignature::new())
            .wrap(Logger::default())
            .service(web::scope("/webhook").configure(webhook::configure_webhooks))
    })
    .bind(ip)?
    .run()
    .await
}
