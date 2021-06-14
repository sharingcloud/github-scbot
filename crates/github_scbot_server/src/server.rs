//! Server module.

use actix_cors::Cors;
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use github_scbot_api::adapter::GithubAPIAdapter;
use github_scbot_conf::{sentry::with_sentry_configuration, Config};
use github_scbot_database::DbPool;
use github_scbot_redis::RedisAdapter;
use sentry_actix::Sentry;
use tracing::info;

use crate::{
    external::{status::set_qa_status, validator::jwt_auth_validator},
    middlewares::VerifySignature,
    webhook::configure_webhook_handlers,
    Result, ServerError,
};

/// App context.
#[derive(Clone)]
pub struct AppContext {
    /// Config.
    pub config: Config,
    /// Database pool.
    pub pool: DbPool,
    /// API adapter
    pub api_adapter: GithubAPIAdapter,
    /// Redis adapter
    pub redis_adapter: RedisAdapter,
}

/// Run bot server.
pub async fn run_bot_server(config: Config, pool: &DbPool) -> Result<()> {
    info!(
        version = env!("CARGO_PKG_VERSION"),
        message = "Starting bot server",
    );

    with_sentry_configuration(&config, || async {
        let config = config.clone();
        let address = get_bind_address(&config);
        run_bot_server_internal(config, pool, address).await
    })
    .await
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server_bind_ip, config.server_bind_port)
}

async fn run_bot_server_internal(
    config: Config,
    pool: &DbPool,
    ip_with_port: String,
) -> Result<()> {
    let app_context = AppContext {
        config: config.clone(),
        pool: pool.clone(),
        api_adapter: GithubAPIAdapter::new(&config).await?,
        redis_adapter: RedisAdapter::new(config.redis_address),
    };

    HttpServer::new(move || {
        App::new()
            .data(app_context.clone())
            .wrap(Sentry::builder().capture_client_errors(true).finish())
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
            .route(
                "/health-check",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .json(serde_json::json!({"message": "Ok"}))
                        .await
                }),
            )
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .json(serde_json::json!({"message": "Welcome on github-scbot !" }))
                        .await
                }),
            )
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                // Display Bad Request response on invalid JSON data
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": err.to_string()
                    })),
                )
                .into()
            }))
    })
    .bind(ip_with_port)?
    .run()
    .await
    .map_err(ServerError::from)
}
