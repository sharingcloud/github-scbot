//! Log configuration.

use tracing_subscriber::EnvFilter;

const DEFAULT_ENV_CONFIG: &str = "warn,github_scbot=debug";

pub fn configure_logging() {
    let use_json = &std::env::var("USE_JSON").unwrap_or_else(|_| "".to_string()) == "1";

    if std::env::var("RUST_LOG").unwrap_or_default().is_empty() {
        std::env::set_var("RUST_LOG", DEFAULT_ENV_CONFIG);
    }

    let s = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env());

    if use_json {
        s.json().init();
    } else {
        s.compact().init();
    }
}
