//! External status handlers.

use actix_web::{error, web, HttpResponse, Result};
use github_scbot_logic::external::set_qa_status_for_pull_requests;
use serde::{Deserialize, Serialize};

use crate::server::AppContext;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QAStatusJson {
    repository_path: String,
    pull_request_numbers: Vec<u64>,
    author: String,
    status: Option<bool>,
}

pub(crate) async fn set_qa_status(
    ctx: web::Data<AppContext>,
    data: web::Json<QAStatusJson>,
) -> Result<HttpResponse> {
    let conn = ctx.pool.get().unwrap();
    set_qa_status_for_pull_requests(
        &ctx.config,
        &conn,
        &data.repository_path,
        &data.pull_request_numbers,
        &data.author,
        data.status,
    )
    .await
    .map_err(|_e| error::ErrorInternalServerError("Error."))?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
