//! API crate.
//!
//! Contains functions to communicate with GitHub API.

pub mod comments;
pub mod errors;
pub mod labels;
pub mod pulls;
pub mod reviews;
pub mod status;

use github_scbot_core::constants::{ENV_API_DISABLE_CLIENT, ENV_GITHUB_API_TOKEN};
use octocrab::Octocrab;

pub use self::errors::{APIError, Result};

/// Get an authenticated GitHub client.
pub async fn get_client() -> Result<Octocrab> {
    let client = Octocrab::builder()
        .personal_token(std::env::var(ENV_GITHUB_API_TOKEN).unwrap())
        .build()?;

    Ok(client)
}

fn is_client_enabled() -> bool {
    std::env::var(ENV_API_DISABLE_CLIENT).ok().is_none()
}
