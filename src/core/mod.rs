//! Core module

use crate::errors::{BotError, Result};

pub fn configure_startup() -> Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");

    tracing_subscriber::fmt::init();

    validate_configuration()
}

fn check_env_var(name: &str) -> Option<String> {
    let entry: String = std::env::var(name).unwrap_or_default();
    if entry == "" {
        None
    } else {
        Some(entry)
    }
}

fn ensure_env_vars(names: &[&str]) -> Result<(), BotError> {
    let mut error = String::new();

    for name in names {
        if check_env_var(name).is_none() {
            error.push('\n');
            error.push_str(&format!("  - Missing env. var.: {}", name));
        }
    }

    if error.is_empty() {
        Ok(())
    } else {
        Err(BotError::ConfigurationError(error))
    }
}

fn validate_configuration() -> Result<()> {
    ensure_env_vars(&[
        crate::api::constants::ENV_GITHUB_API_TOKEN,
        crate::database::constants::ENV_DATABASE_URL,
        crate::server::constants::ENV_BIND_IP,
        crate::server::constants::ENV_BIND_PORT,
        crate::webhook::constants::ENV_BOT_USERNAME,
    ])
}
