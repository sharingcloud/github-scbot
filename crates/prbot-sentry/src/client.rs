use std::{future::Future, str::FromStr};

use prbot_config::Config;
use sentry::{integrations::debug_images::DebugImagesIntegration, types::Dsn, ClientOptions};
use tracing::info;

/// Configure Sentry integration by wrapping a function.
pub async fn with_sentry_configuration<T, Fut, E>(config: &Config, func: T) -> Result<(), E>
where
    T: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), E>>,
{
    let _guard = {
        if config.sentry.url.is_empty() {
            None
        } else {
            info!("Sentry integration enabled.");

            // Enable backtraces
            std::env::set_var("RUST_BACKTRACE", "1");

            let mut options =
                ClientOptions::new().add_integration(DebugImagesIntegration::default());

            options.dsn = Some(Dsn::from_str(&config.sentry.url).unwrap());
            options.default_integrations = true;
            options.in_app_exclude.push("actix");
            options.in_app_exclude.push("sentry");
            options.in_app_exclude.push("tokio");
            options.release = Some(config.version.to_string().into());
            options.send_default_pii = true;
            options.attach_stacktrace = true;
            options.traces_sample_rate = config.sentry.traces_sample_rate;
            options.debug = false;

            let init = sentry::init(options);
            Some(init)
        }
    };

    func().await
}
