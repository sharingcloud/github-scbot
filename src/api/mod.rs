//! API module

use actix_web::web;
use octocrab::Octocrab;

pub mod comments;
pub mod constants;
mod handlers;
pub mod labels;
pub mod pulls;
pub mod reviews;
pub mod status;

use crate::errors::{BotError, Result};
use constants::{ENV_API_DEBUG_MODE, ENV_GITHUB_API_TOKEN};

pub async fn get_client() -> Result<Octocrab> {
    let client = Octocrab::builder()
        .personal_token(std::env::var(ENV_GITHUB_API_TOKEN).map_err(|_e| {
            BotError::ConfigurationError(format!("Missing {} env var", ENV_GITHUB_API_TOKEN))
        })?)
        .build()?;

    Ok(client)
}

pub fn configure_debug_api(cfg: &mut web::ServiceConfig) {
    if std::env::var(ENV_API_DEBUG_MODE).ok().is_some() {
        cfg.service(
            web::resource("/welcome-comment").route(web::post().to(handlers::welcome_comment)),
        );
    }
}
