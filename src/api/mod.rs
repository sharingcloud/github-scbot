//! API module.
//!
//! Contains functions to communicate with GitHub API.

pub mod comments;
pub mod constants;
pub mod errors;
pub mod labels;
pub mod pulls;
pub mod reviews;
pub mod status;

use octocrab::Octocrab;

use self::constants::ENV_GITHUB_API_TOKEN;
pub use self::errors::{APIError, Result};

/// Get an authenticated GitHub client.
pub async fn get_client() -> Result<Octocrab> {
    let client = Octocrab::builder()
        .personal_token(std::env::var(ENV_GITHUB_API_TOKEN).unwrap())
        .build()?;

    Ok(client)
}
