use std::{future::Future, str::FromStr};

use github_scbot_config::Config;
use sentry::{types::Dsn, ClientOptions};
use tracing::info;

/// Configure Sentry integration by wrapping a function.
pub async fn with_sentry_configuration<T, Fut, E>(config: &Config, func: T) -> Result<(), E>
where
    T: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), E>>,
{
    let _guard = {
        if config.sentry_url.is_empty() {
            None
        } else {
            info!("Sentry integration enabled.");

            // Enable backtraces
            std::env::set_var("RUST_BACKTRACE", "1");

            let mut options = ClientOptions::new();
            options.dsn = Some(Dsn::from_str(&config.sentry_url).unwrap());
            options.default_integrations = true;
            options.in_app_exclude.push("actix_cors");
            options.in_app_exclude.push("actix_http");
            options.in_app_exclude.push("actix_rt");
            options.in_app_exclude.push("actix_server");
            options.in_app_exclude.push("actix_service");
            options.in_app_exclude.push("actix_web");
            options.in_app_exclude.push("actix_web_httpauth");
            options.in_app_exclude.push("sentry_actix");
            options.in_app_exclude.push("sentry_backtrace");
            options.in_app_exclude.push("sentry_core");
            options.in_app_exclude.push("tokio");
            options.release = Some(env!("CARGO_PKG_VERSION").into());
            options.send_default_pii = true;
            options.attach_stacktrace = true;
            options.traces_sample_rate = config.sentry_traces_sample_rate;
            options.debug = false;

            let init = sentry::init(options);
            Some(init)
        }
    };

    func().await
}
