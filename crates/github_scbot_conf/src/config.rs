//! Config module.

use std::env;

/// Bot configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Bot username.
    pub bot_username: String,
    /// Database URL.
    pub database_url: String,
    /// Database pool size.
    pub database_pool_size: u32,
    /// Default merge strategy.
    pub default_merge_strategy: String,
    /// Default needed reviewers count.
    pub default_needed_reviewers_count: u64,
    /// Default PR title validation regex.
    pub default_pr_title_validation_regex: String,
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
    /// Redis address.
    pub redis_address: String,
    /// Sentry URL.
    pub sentry_url: String,
    /// Server bind IP.
    pub server_bind_ip: String,
    /// Server bind port.
    pub server_bind_port: u16,
    /// Disable webhook signature verification.
    pub server_disable_webhook_signature: bool,
    /// Enable history tracking.
    pub server_enable_history_tracking: bool,
    /// Enable welcome coments.
    pub server_enable_welcome_comments: bool,
    /// Tenor API key.
    pub tenor_api_key: String,
    /// Test database URL.
    pub test_database_url: String,
    /// Test debug mode
    pub test_debug_mode: bool,
}

impl Config {
    /// Create configuration from environment.
    pub fn from_env() -> Config {
        Config {
            bot_username: env_to_str("BOT_USERNAME", "bot"),
            database_url: env_to_str("DATABASE_URL", ""),
            database_pool_size: env_to_u32("BOT_DATABASE_POOL_SIZE", 20),
            default_merge_strategy: env_to_str("BOT_DEFAULT_MERGE_STRATEGY", "merge"),
            default_needed_reviewers_count: env_to_u64("BOT_DEFAULT_NEEDED_REVIEWERS_COUNT", 2),
            default_pr_title_validation_regex: env_to_str(
                "BOT_DEFAULT_PR_TITLE_VALIDATION_REGEX",
                "",
            ),
            github_api_token: env_to_str("BOT_GITHUB_API_TOKEN", ""),
            github_app_id: env_to_u64("BOT_GITHUB_APP_ID", 0),
            github_app_installation_id: env_to_u64("BOT_GITHUB_APP_INSTALLATION_ID", 0),
            github_app_private_key: env_to_str("BOT_GITHUB_APP_PRIVATE_KEY", ""),
            github_webhook_secret: env_to_str("BOT_GITHUB_WEBHOOK_SECRET", ""),
            redis_address: env_to_str("BOT_REDIS_ADDRESS", ""),
            sentry_url: env_to_str("BOT_SENTRY_URL", ""),
            server_bind_ip: env_to_str("BOT_SERVER_BIND_IP", "127.0.0.1"),
            server_bind_port: env_to_u16("BOT_SERVER_BIND_IP", 8008),
            server_disable_webhook_signature: env_to_bool(
                "BOT_SERVER_DISABLE_WEBHOOK_SIGNATURE",
                false,
            ),
            server_enable_history_tracking: env_to_bool(
                "BOT_SERVER_ENABLE_HISTORY_TRACKING",
                false,
            ),
            server_enable_welcome_comments: env_to_bool(
                "BOT_SERVER_ENABLE_WELCOME_COMMENTS",
                false,
            ),
            tenor_api_key: env_to_str("BOT_TENOR_API_KEY", ""),
            test_database_url: env_to_str("TEST_DATABASE_URL", ""),
            test_debug_mode: env_to_bool("TEST_DEBUG_MODE", false),
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

fn env_to_u32(name: &str, default: u32) -> u32 {
    env::var(name)
        .map(|e| e.parse().unwrap_or(default))
        .unwrap_or(default)
}

fn env_to_bool(name: &str, default: bool) -> bool {
    env::var(name).map(|e| !e.is_empty()).unwrap_or(default)
}

fn env_to_str(name: &str, default: &str) -> String {
    env::var(name)
        .unwrap_or_else(|_e| default.to_string())
        .replace("\\n", "\n")
}
