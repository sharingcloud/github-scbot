//! Webhook module

use actix_web::web;

pub mod constants;
pub mod errors;
mod handlers;
pub mod logic;
mod middlewares;
mod utils;

#[cfg(test)]
mod tests;

pub use middlewares::VerifySignature;

pub fn configure_webhooks(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(handlers::event_handler)));
}
