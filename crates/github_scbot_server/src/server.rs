//! Server module.

use actix_cors::Cors;
use actix_web::{middleware::Logger, rt, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use github_scbot_conf::Config;
use github_scbot_database::{establish_connection, DbPool};
use sentry_actix::Sentry;
use tracing::{error, info};

use crate::{
    external::{status::set_qa_status, validator::jwt_auth_validator},
    middlewares::VerifySignature,
    sentry_utils::with_sentry_configuration,
    webhook::configure_webhook_handlers,
    Result,
};

/// App context.
#[derive(Clone)]
pub struct AppContext {
    /// Config.
    pub config: Config,
    /// Database pool.
    pub pool: DbPool,
}

/// Run bot server.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn run_bot_server(config: Config) -> Result<()> {
    info!("Starting bot server v{} ...", env!("CARGO_PKG_VERSION"));

    with_sentry_configuration(config.clone(), || {
        let mut sys = rt::System::new("app");
        let address = get_bind_address(&config);
        sys.block_on(run_bot_server_internal(config, address))
    })
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server_bind_ip, config.server_bind_port)
}

async fn run_bot_server_internal(config: Config, ip_with_port: String) -> Result<()> {
    if let Ok(pool) = establish_connection(&config) {
        let app_context = AppContext {
            config: config.clone(),
            pool: pool.clone(),
        };

        HttpServer::new(move || {
            App::new()
                .data(app_context.clone())
                .wrap(Sentry::new())
                .wrap(Logger::default())
                .service(
                    web::scope("/external")
                        .wrap(HttpAuthentication::bearer(jwt_auth_validator))
                        .wrap(Cors::permissive())
                        .route("/set-qa-status", web::post().to(set_qa_status)),
                )
                .service(
                    web::scope("/webhook")
                        .wrap(VerifySignature::new(&app_context.config))
                        .configure(configure_webhook_handlers),
                )
                .route("/", web::get().to(|| async { "Welcome on github-scbot !" }))
        })
        .bind(ip_with_port)?
        .run()
        .await
        .map_err(Into::into)
    } else {
        error!("Error while establishing connection to database.");
        std::process::exit(1);
    }
}
