//! Server module.

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    error,
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use prbot_config::Config;
use prbot_core::{CoreContext, CoreModule};
use prbot_database_interface::DbService;
use prbot_database_pg::{DbPool, PostgresDb};
use prbot_ghapi_interface::ApiService;
use prbot_lock_interface::LockService;
use sentry_actix::Sentry;
use tracing::info;

use crate::{
    admin::{accounts_list, external_accounts_list, pull_request_rules_create, pull_request_rules_delete, repositories_list, validator::admin_jwt_auth_validator},
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
    /// Core module.
    pub core_module: CoreModule,
    /// Database pool.
    pub db_service: Box<dyn DbService>,
    /// API adapter
    pub api_service: Box<dyn ApiService>,
    /// Redis adapter
    pub lock_service: Box<dyn LockService>,
}

impl AppContext {
    /// Create new app context.
    pub fn new(config: Config, core_module: CoreModule, pool: DbPool) -> Self {
        Self {
            config: config.clone(),
            core_module,
            db_service: Box::new(PostgresDb::new(pool)),
            api_service: Box::new(MetricsApiService::new(config.clone())),
            lock_service: Box::new(MetricsRedisService::new(&config.lock.redis.address)),
        }
    }

    /// Create new app context using adapters.
    pub fn new_with_adapters(
        config: Config,
        core_module: CoreModule,
        db_service: Box<dyn DbService + Send + Sync>,
        api_service: Box<dyn ApiService + Send + Sync>,
        lock_service: Box<dyn LockService + Send + Sync>,
    ) -> Self {
        Self {
            config,
            core_module,
            db_service,
            api_service,
            lock_service,
        }
    }

    /// Convert the context for the core module.
    pub fn as_core_context(&self) -> CoreContext {
        CoreContext {
            config: &self.config,
            api_service: self.api_service.as_ref(),
            db_service: self.db_service.as_ref(),
            lock_service: self.lock_service.as_ref(),
            core_module: &self.core_module,
        }
    }
}

/// Build Actix app.
pub fn build_actix_app(
    context: Data<AppContext>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let prometheus = build_metrics_handler();

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
            web::scope("/admin")
                .wrap(HttpAuthentication::bearer(admin_jwt_auth_validator))
                .wrap(Cors::permissive())
                .route("/accounts/", web::get().to(accounts_list))
                .route("/repositories/", web::get().to(repositories_list))
                .route("/repositories/{repository_id}/pull-request-rules/", web::post().to(pull_request_rules_create))
                .route("/repositories/{repository_id}/pull-request-rules/{rule_name}/", web::delete().to(pull_request_rules_delete))
                .route("/external-accounts/", web::get().to(external_accounts_list))
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
                HttpResponse::Ok().json(serde_json::json!({"message": "Welcome on prbot!" }))
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
}

/// Run bot server.
pub async fn run_bot_server(context: AppContext) -> Result<()> {
    let address = get_bind_address(&context.config);

    info!(
        version = context.config.version,
        address = %address,
        message = "Starting bot server",
    );

    run_bot_server_internal(address, context).await
}

fn get_bind_address(config: &Config) -> String {
    format!("{}:{}", config.server.bind_ip, config.server.bind_port)
}

async fn run_bot_server_internal(ip_with_port: String, context: AppContext) -> Result<()> {
    let context = Data::new(context);
    let cloned_context = context.clone();

    let mut server = HttpServer::new(move || build_actix_app(context.clone()));

    if let Some(workers) = cloned_context.config.server.workers_count {
        server = server.workers(workers as usize);
    }

    server
        .bind(ip_with_port)
        .map_err(|e| ServerError::IoError { source: e })?
        .run()
        .await
        .map_err(|e| ServerError::IoError { source: e })
}
