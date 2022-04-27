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
use actix_web_prom::PrometheusMetricsBuilder;
use github_scbot_conf::Config;
use github_scbot_database2::{DbPool, DbService, DbServiceImplPool};
use github_scbot_ghapi::adapter::{GithubApiService, ApiService};
use github_scbot_redis::{RedisService, RedisServiceImpl};
use github_scbot_sentry::{actix::Sentry, with_sentry_configuration};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{
    debug::configure_debug_handlers,
    external::{status::set_qa_status, validator::jwt_auth_validator},
    middlewares::VerifySignature,
    webhook::configure_webhook_handlers,
    Result, ServerError,
};

/// App context.
pub struct AppContext {
    /// Config.
    pub config: Config,
    /// Database pool.
    pub db_adapter: Box<dyn DbService>,
    /// API adapter
    pub api_adapter: Box<dyn ApiService>,
    /// Redis adapter
    pub redis_adapter: Box<dyn RedisService>,
}

impl AppContext {
    /// Create new app context.
    pub fn new(config: Config, pool: DbPool) -> Self {
        Self {
            config: config.clone(),
            db_adapter: Box::new(DbServiceImplPool::new(pool)),
            api_adapter: Box::new(GithubApiService::new(config.clone())),
            redis_adapter: Box::new(RedisServiceImpl::new(&config.redis_address)),
        }
    }

    /// Create new app context using adapters.
    pub fn new_with_adapters(
        config: Config,
        db_adapter: Box<dyn DbService>,
        api_adapter: Box<dyn ApiService>,
        redis_adapter: Box<dyn RedisService>,
    ) -> Self {
        Self {
            config,
            db_adapter,
            api_adapter,
            redis_adapter,
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

    with_sentry_configuration(&context.config.sentry_url.clone(), || async {
        run_bot_server_internal(address, context).await
    })
    .await
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server_bind_ip, config.server_bind_port)
}

async fn run_bot_server_internal(ip_with_port: String, context: AppContext) -> Result<()> {
    let context = Data::new(Arc::new(context));
    let cloned_context = context.clone();
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let mut server = HttpServer::new(move || {
        let mut app = App::new()
            .app_data(context.clone())
            .wrap(prometheus.clone())
            .wrap(Sentry::new())
            .wrap(Logger::default())
            .wrap(TracingLogger::default())
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
            .route(
                "/health-check",
                web::get()
                    .to(|| async { HttpResponse::Ok().json(serde_json::json!({"message": "Ok"})) }),
            )
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
        .bind(ip_with_port)?
        .run()
        .await
        .map_err(ServerError::from)
}
