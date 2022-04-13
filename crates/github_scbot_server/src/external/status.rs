//! External status handlers.

use std::{sync::Arc, str::FromStr};

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_logic::external::set_qa_status_for_pull_requests;
use github_scbot_sentry::{sentry, WrapEyre};
use github_scbot_types::repository::RepositoryPath;
use serde::{Deserialize, Serialize};

use crate::{external::validator::extract_account_from_auth, server::AppContext, ServerError};

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
    let target_account = extract_account_from_auth(&mut *ctx.db_adapter.external_accounts(), &auth)
        .await
        .map_err(WrapEyre::to_http_error)?;

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(target_account.username().into()),
            ..sentry::User::default()
        }));
    });

    // TODO: CAN EXPLODE
    let repo_path = RepositoryPath::from_str(&data.repository_path).unwrap();

    set_qa_status_for_pull_requests(
        ctx.api_adapter.as_ref(),
        ctx.db_adapter.as_ref(),
        ctx.redis_adapter.as_ref(),
        &target_account,
        repo_path,
        &data.pull_request_numbers,
        &data.author,
        data.status,
    )
    .await
    .map_err(ServerError::from)
    .map_err(WrapEyre::to_http_error)?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
