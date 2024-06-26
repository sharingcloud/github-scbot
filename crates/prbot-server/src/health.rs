use actix_http::StatusCode;
use actix_web::{web, HttpResponse, Responder};

use crate::server::AppContext;

pub async fn health_check_route(ctx: web::Data<AppContext>) -> impl Responder {
    let pg_status = ctx.db_service.health_check().await.is_ok();
    let redis_status = ctx.lock_service.health_check().await.is_ok();
    let all_good = pg_status && redis_status;
    let status_code = if all_good {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    HttpResponse::build(status_code).json(serde_json::json!({
        "postgresql": pg_status,
        "redis": redis_status,
    }))
}
