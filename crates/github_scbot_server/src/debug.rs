use actix_web::{web, HttpResponse, Result as ActixResult};
use github_scbot_ghapi::ApiError;
use github_scbot_logic::LogicError;
use github_scbot_sentry::WrapEyre;

use crate::ServerError;

pub fn configure_debug_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("panic").route(web::get().to(panic_route)));
    cfg.service(web::resource("error").route(web::get().to(error_route)));
    cfg.service(web::resource("error-nest").route(web::get().to(error_route_nest)));
}

async fn error_route() -> ActixResult<HttpResponse> {
    will_error().await.map_err(WrapEyre::to_http_error)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "ok"})))
}

async fn error_route_nest() -> ActixResult<HttpResponse> {
    will_error_nest().await.map_err(WrapEyre::to_http_error)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "ok"})))
}

async fn panic_route() -> ActixResult<HttpResponse> {
    panic!("Oh noes, a panic.")
}

async fn will_error() -> Result<(), ServerError> {
    Err(ServerError::ThreadpoolError)
}

async fn _will_error_nest_api() -> Result<(), ApiError> {
    Err(ApiError::MergeError("Nope.".into()))
}

async fn _will_error_nest_logic() -> Result<(), LogicError> {
    _will_error_nest_api().await.map_err(Into::into)
}

async fn will_error_nest() -> Result<(), ServerError> {
    _will_error_nest_logic().await.map_err(Into::into)
}
