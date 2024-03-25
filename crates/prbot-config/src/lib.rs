//! Config module.

mod drivers;

use std::{
    env,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

pub use drivers::{ApiDriver, DatabaseDriver, DriverError, LockDriver};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database driver.
    pub driver: DatabaseDriver,
    /// Postgres options.
    pub pg: DatabasePgConfig,
}

#[derive(Debug, Clone)]
pub struct DatabasePgConfig {
    /// Database URL.
    pub url: String,
    /// Database pool size.
    pub pool_size: u32,
    /// Database connection timeout (in seconds)
    pub connection_timeout: u32,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API driver.
    pub driver: ApiDriver,
    /// GitHub options.
    pub github: ApiGitHubConfig,
}

#[derive(Debug, Clone)]
pub struct ApiGitHubConfig {
    /// GitHub API connect timeout.
    pub connect_timeout: u64,
    /// GitHub API root URL.
    pub root_url: String,
    /// GitHub API personal token.
    pub token: String,
    /// GitHub App ID.
    pub app_id: u64,
    /// GitHub App installation ID.
    pub app_installation_id: u64,
    /// GitHub App private key.
    pub app_private_key: String,
}

#[derive(Debug, Clone)]
pub struct LockConfig {
    /// Lock driver.
    pub driver: LockDriver,
    /// Redis options.
    pub redis: LockRedisConfig,
}

#[derive(Debug, Clone)]
pub struct LockRedisConfig {
    /// Redis address.
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// Sentry URL.
    pub url: String,
    /// Traces sample rate (between 0 and 1) for Sentry
    pub traces_sample_rate: f32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Use bunyan logging.
    pub use_bunyan: bool,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind IP.
    pub bind_ip: String,
    /// Server bind port.
    pub bind_port: u16,
    /// Server workers count.
    pub workers_count: Option<u16>,
    /// Server webhook secret.
    pub webhook_secret: String,
    /// Disable webhook signature verification.
    pub disable_webhook_signature: bool,
    /// Enable welcome coments.
    pub enable_welcome_comments: bool,
}

/// Bot configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Bot username.
    pub name: String,
    /// Database options.
    pub database: DatabaseConfig,
    /// Default merge strategy.
    pub default_merge_strategy: String,
    /// Default needed reviewers count.
    pub default_needed_reviewers_count: u64,
    /// Default PR title validation regex.
    pub default_pr_title_validation_regex: String,
    /// API options.
    pub api: ApiConfig,
    /// Logging options.
    pub logging: LoggingConfig,
    /// Lock options.
    pub lock: LockConfig,
    /// Sentry options.
    pub sentry: SentryConfig,
    /// Server options.
    pub server: ServerConfig,
    /// Tenor API key.
    pub tenor_api_key: String,
    /// Test debug mode
    pub test_debug_mode: bool,
    /// Random seed
    pub random_seed: u64,
    /// App version
    pub version: String,
}

impl Config {
    /// Create configuration from environment.
    pub fn from_env(version: String) -> Config {
        Config {
            name: env_to_str("BOT_NAME", "bot"),
            database: DatabaseConfig {
                driver: DatabaseDriver::from_str(&env_to_str("BOT_DATABASE_DRIVER", "pg")).unwrap(),
                pg: DatabasePgConfig {
                    url: env_to_str("BOT_DATABASE_PG_URL", ""),
                    pool_size: env_to_u32("BOT_DATABASE_PG_POOL_SIZE", 20),
                    connection_timeout: env_to_u32("BOT_DATABASE_PG_CONNECTION_TIMEOUT", 5),
                },
            },
            default_merge_strategy: env_to_str("BOT_DEFAULT_MERGE_STRATEGY", "merge"),
            default_needed_reviewers_count: env_to_u64("BOT_DEFAULT_NEEDED_REVIEWERS_COUNT", 2),
            default_pr_title_validation_regex: env_to_str(
                "BOT_DEFAULT_PR_TITLE_VALIDATION_REGEX",
                "",
            ),
            api: ApiConfig {
                driver: ApiDriver::from_str(&env_to_str("BOT_API_DRIVER", "github")).unwrap(),
                github: ApiGitHubConfig {
                    connect_timeout: env_to_u64("BOT_API_GITHUB_CONNECT_TIMEOUT", 5000),
                    root_url: env_to_str("BOT_API_GITHUB_ROOT_URL", "https://api.github.com"),
                    token: env_to_str("BOT_API_GITHUB_TOKEN", ""),
                    app_id: env_to_u64("BOT_API_GITHUB_APP_ID", 0),
                    app_installation_id: env_to_u64("BOT_API_GITHUB_APP_INSTALLATION_ID", 0),
                    app_private_key: env_to_str("BOT_API_GITHUB_APP_PRIVATE_KEY", ""),
                },
            },
            logging: LoggingConfig {
                use_bunyan: env_to_bool("BOT_LOGGING_USE_BUNYAN", false),
            },
            lock: LockConfig {
                driver: LockDriver::from_str(&env_to_str("BOT_LOCK_DRIVER", "redis")).unwrap(),
                redis: LockRedisConfig {
                    address: env_to_str("BOT_LOCK_REDIS_ADDRESS", "redis://localhost"),
                },
            },
            sentry: SentryConfig {
                url: env_to_str("BOT_SENTRY_URL", ""),
                traces_sample_rate: env_to_f32("BOT_SENTRY_TRACES_SAMPLE_RATE", 0.0),
            },
            server: ServerConfig {
                bind_ip: env_to_str("BOT_SERVER_BIND_IP", "127.0.0.1"),
                bind_port: env_to_u16("BOT_SERVER_BIND_PORT", 8008),
                workers_count: env_to_optional_u16("BOT_SERVER_WORKERS_COUNT", None),
                webhook_secret: env_to_str("BOT_SERVER_WEBHOOK_SECRET", ""),
                disable_webhook_signature: env_to_bool(
                    "BOT_SERVER_DISABLE_WEBHOOK_SIGNATURE",
                    false,
                ),
                enable_welcome_comments: env_to_bool("BOT_SERVER_ENABLE_WELCOME_COMMENTS", false),
            },
            tenor_api_key: env_to_str("BOT_TENOR_API_KEY", ""),
            test_debug_mode: env_to_bool("BOT_TEST_DEBUG_MODE", false),
            random_seed: env_to_u64("BOT_RANDOM_SEED", random_seed()),
            version,
        }
    }

    pub fn from_env_no_version() -> Self {
        Self::from_env("0.0.0".into())
    }
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn env_to_u16(name: &str, default: u16) -> u16 {
    env::var(name)
        .map(|e| e.parse().unwrap_or(default))
        .unwrap_or(default)
}

fn env_to_optional_u16(name: &str, default: Option<u16>) -> Option<u16> {
    env::var(name)
        .map(|e| e.parse::<u16>().map(Some).unwrap_or(default))
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

fn env_to_f32(name: &str, default: f32) -> f32 {
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
