//! Webhook module.

#![warn(missing_docs)]
#![warn(clippy::all)]

use actix_web::web;

pub mod constants;
pub mod errors;
mod handlers;
pub mod middlewares;
pub mod sentry_utils;
pub mod server;
pub mod utils;

#[cfg(test)]
mod tests;

pub use errors::{Result, ServerError};
pub use middlewares::VerifySignature;

/// Configure webhooks.
///
/// # Arguments
///
/// * `cfg` - Actix service config
pub fn configure_webhooks(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(handlers::event_handler)));
}
