//! External status handlers.

use std::sync::Arc;

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_logic::external::set_qa_status_for_pull_requests;
use sentry_actix::eyre::WrapEyre;
use serde::{Deserialize, Serialize};

use crate::{external::validator::extract_account_from_auth, server::AppContext};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QaStatusJson {
    repository_path: String,
    pull_request_numbers: Vec<u64>,
    author: String,
    status: Option<bool>,
}

pub(crate) async fn set_qa_status(
    ctx: web::Data<Arc<AppContext>>,
    data: web::Json<QaStatusJson>,
    auth: BearerAuth,
) -> Result<HttpResponse> {
    let target_account = extract_account_from_auth(ctx.db_adapter.as_ref(), &auth)
        .await
        .map_err(WrapEyre::bad_request)?;

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(target_account.username.clone()),
            ..sentry::User::default()
        }));
    });

    set_qa_status_for_pull_requests(
        ctx.api_adapter.as_ref(),
        ctx.db_adapter.as_ref(),
        ctx.redis_adapter.as_ref(),
        &target_account,
        &data.repository_path,
        &data.pull_request_numbers,
        &data.author,
        data.status,
    )
    .await
    .map_err(WrapEyre::bad_request)?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
