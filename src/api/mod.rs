//! API module

use actix_web::web;
use eyre::Result;
use octocrab::Octocrab;

pub mod comments;
mod constants;
mod handlers;
pub mod labels;

use constants::{ENV_API_DEBUG_MODE, ENV_GITHUB_API_TOKEN};

pub async fn get_client() -> Result<Octocrab> {
    Octocrab::builder()
        .personal_token(std::env::var(ENV_GITHUB_API_TOKEN)?)
        .build()
        .map_err(Into::into)
}

pub fn configure_debug_api(cfg: &mut web::ServiceConfig) {
    if std::env::var(ENV_API_DEBUG_MODE).ok().is_some() {
        cfg.service(
            web::resource("/welcome-comment").route(web::post().to(handlers::welcome_comment)),
        );
    }
}
