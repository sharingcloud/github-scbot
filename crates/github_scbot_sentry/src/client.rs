use std::{future::Future, str::FromStr};

use sentry::{types::Dsn, ClientOptions};
use tracing::info;

/// Configure Sentry integration by wrapping a function.
pub async fn with_sentry_configuration<T, Fut, E>(sentry_url: &str, func: T) -> Result<(), E>
where
    T: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), E>>,
{
    let _guard = {
        if sentry_url.is_empty() {
            None
        } else {
            info!("Sentry integration enabled.");

            // Enable backtraces
            std::env::set_var("RUST_BACKTRACE", "1");

            // Ignore eyre modules
            let mut options = ClientOptions::new();
            options.dsn = Some(Dsn::from_str(sentry_url).unwrap());
            options.default_integrations = true;
            options.in_app_exclude.push("actix_cors");
            options.in_app_exclude.push("actix_http");
            options.in_app_exclude.push("actix_rt");
            options.in_app_exclude.push("actix_server");
            options.in_app_exclude.push("actix_service");
            options.in_app_exclude.push("actix_web");
            options.in_app_exclude.push("actix_web_httpauth");
            options.in_app_exclude.push("eyre");
            options.in_app_exclude.push("futures_util");
            options.in_app_exclude.push("sentry_actix");
            options.in_app_exclude.push("stable_eyre");
            options.in_app_exclude.push("tokio");
            options.release = sentry::release_name!();
            options.send_default_pii = true;
            // options.attach_stacktrace = true;
            options.debug = true;

            let init = sentry::init(options);
            Some(init)
        }
    };

    func().await
}
