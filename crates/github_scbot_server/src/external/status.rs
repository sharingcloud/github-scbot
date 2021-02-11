//! External status handlers.

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_logic::external::set_qa_status_for_pull_requests;
use serde::{Deserialize, Serialize};

use crate::{external::validator::extract_account_from_auth, server::AppContext, ServerError};

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
    auth: BearerAuth,
) -> Result<HttpResponse> {
    let conn = ctx.pool.get().unwrap();
    let target_account = extract_account_from_auth(&conn, &auth)?;

    set_qa_status_for_pull_requests(
        &ctx.config,
        &conn,
        &target_account,
        &data.repository_path,
        &data.pull_request_numbers,
        &data.author,
        data.status,
    )
    .await
    .map_err(ServerError::LogicError)?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
