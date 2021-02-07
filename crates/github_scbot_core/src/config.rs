//! Config module.

use std::env;

/// Bot configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Disable GitHub client.
    pub api_disable_client: bool,
    /// Bot username.
    pub bot_username: String,
    /// Database URL.
    pub database_url: String,
    /// GitHub API personal token.
    pub github_api_token: String,
    /// GitHub App ID.
    pub github_app_id: u64,
    /// GitHub App installation ID.
    pub github_app_installation_id: u64,
    /// GitHub App private key.
    pub github_app_private_key: String,
    /// GitHub webhook secret.
    pub github_webhook_secret: String,
    /// Sentry URL.
    pub sentry_url: String,
    /// Server bind IP.
    pub server_bind_ip: String,
    /// Server bind port.
    pub server_bind_port: u16,
    /// Disable webhook signature verification.
    pub server_disable_webhook_signature: bool,
    /// Enable welcome coments.
    pub server_enable_welcome_comments: bool,
    /// Test database URL.
    pub test_database_url: String,
}

impl Config {
    /// Create configuration from environment.
    pub fn from_env() -> Config {
        Config {
            api_disable_client: env_to_bool("BOT_API_DISABLE_CLIENT", false),
            bot_username: env_to_str("BOT_USERNAME", "bot"),
            database_url: env_to_str("DATABASE_URL", ""),
            github_api_token: env_to_str("BOT_GITHUB_API_TOKEN", ""),
            github_app_id: env_to_u64("BOT_GITHUB_APP_ID", 0),
            github_app_installation_id: env_to_u64("BOT_GITHUB_APP_INSTALLATION_ID", 0),
            github_app_private_key: env_to_str("BOT_GITHUB_APP_PRIVATE_KEY", ""),
            github_webhook_secret: env_to_str("BOT_GITHUB_WEBHOOK_SECRET", ""),
            sentry_url: env_to_str("BOT_SENTRY_URL", ""),
            server_bind_ip: env_to_str("BOT_SERVER_BIND_IP", "127.0.0.1"),
            server_bind_port: env_to_u16("BOT_SERVER_BIND_IP", 8008),
            server_disable_webhook_signature: env_to_bool(
                "BOT_SERVER_DISABLE_WEBHOOK_SIGNATURE",
                false,
            ),
            server_enable_welcome_comments: env_to_bool(
                "BOT_SERVER_ENABLE_WELCOME_COMMENTS",
                false,
            ),
            test_database_url: env_to_str("TEST_DATABASE_URL", ""),
        }
    }
}

fn env_to_u16(name: &str, default: u16) -> u16 {
    env::var(name)
        .map(|e| e.parse().unwrap_or(default))
        .unwrap_or(default)
}

fn env_to_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .map(|e| e.parse().unwrap_or(default))
        .unwrap_or(default)
}

fn env_to_bool(name: &str, default: bool) -> bool {
    env::var(name).map(|e| !e.is_empty()).unwrap_or(default)
}

fn env_to_str(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_e| default.to_string())
}
