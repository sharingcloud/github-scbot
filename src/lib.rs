//! SC Bot library

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_wildcard_for_single_variants,
    clippy::future_not_send,
    clippy::pub_enum_variant_names
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

mod api;
mod database;
mod webhook;

/// Run bot server.
///
/// # Arguments
///
/// * `ip` - Bind IP address
///
/// # Errors
///
/// - `HttpServer` bind error
///
pub async fn run_server(ip: &str) -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    if let Ok(pool) = database::establish_connection() {
        HttpServer::new(move || {
            App::new()
                .data(pool.clone())
                .wrap(webhook::VerifySignature::new())
                .wrap(Logger::default())
                .service(web::scope("/webhook").configure(webhook::configure_webhooks))
                .service(web::scope("/debug").configure(api::configure_debug_api))
                .route("/", web::get().to(|| async { "Welcome on SC Bot!" }))
        })
        .bind(ip)?
        .run()
        .await
    } else {
        eprintln!("Error while establishing connection to database.");
        std::process::exit(1);
    }
}
