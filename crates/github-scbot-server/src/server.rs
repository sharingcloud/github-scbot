//! Server module.

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    error,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_database_pg::{DbPool, PostgresDb};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;
use sentry_actix::Sentry;
use tracing::info;

use crate::{
    debug::configure_debug_handlers,
    external::{status::set_qa_status, validator::jwt_auth_validator},
    ghapi::MetricsApiService,
    health::health_check_route,
    metrics::build_metrics_handler,
    middlewares::VerifySignature,
    redis::MetricsRedisService,
    webhook::configure_webhook_handlers,
    Result, ServerError,
};

/// App context.
pub struct AppContext {
    /// Config.
    pub config: Config,
    /// Database pool.
    pub db_service: Box<dyn DbService>,
    /// API adapter
    pub api_service: Box<dyn ApiService>,
    /// Redis adapter
    pub lock_service: Box<dyn LockService>,
}

impl AppContext {
    /// Create new app context.
    pub fn new(config: Config, pool: DbPool) -> Self {
        Self {
            config: config.clone(),
            db_service: Box::new(PostgresDb::new(pool)),
            api_service: Box::new(MetricsApiService::new(config.clone())),
            lock_service: Box::new(MetricsRedisService::new(&config.redis_address)),
        }
    }

    /// Create new app context using adapters.
    pub fn new_with_adapters(
        config: Config,
        db_service: Box<dyn DbService>,
        api_service: Box<dyn ApiService>,
        lock_service: Box<dyn LockService>,
    ) -> Self {
        Self {
            config,
            db_service,
            api_service,
            lock_service,
        }
    }
}

/// Run bot server.
pub async fn run_bot_server(context: AppContext) -> Result<()> {
    let address = get_bind_address(&context.config);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        address = %address,
        message = "Starting bot server",
    );

    run_bot_server_internal(address, context).await
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server_bind_ip, config.server_bind_port)
}

async fn run_bot_server_internal(ip_with_port: String, context: AppContext) -> Result<()> {
    let context = Data::new(Arc::new(context));
    let cloned_context = context.clone();
    let prometheus = build_metrics_handler();

    let mut server = HttpServer::new(move || {
        let mut app = App::new()
            .app_data(context.clone())
            .wrap(prometheus.clone())
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
                    .wrap(VerifySignature::new(&context.config))
                    .configure(configure_webhook_handlers),
            )
            .route("/health", web::get().to(health_check_route))
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .json(serde_json::json!({"message": "Welcome on github-scbot !" }))
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
            }));

        if context.config.test_debug_mode {
            app = app.service(web::scope("/debug").configure(configure_debug_handlers));
        }

        app
    });

    if let Some(workers) = cloned_context.config.server_workers_count {
        server = server.workers(workers as usize);
    }

    server
        .bind(ip_with_port)
        .map_err(|e| ServerError::IoError { source: e })?
        .run()
        .await
        .map_err(|e| ServerError::IoError { source: e })
}
