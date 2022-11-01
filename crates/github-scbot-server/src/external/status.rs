//! External status handlers.

use std::{str::FromStr, sync::Arc};

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_core::sentry::sentry;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_core::types::status::QaStatus;
use github_scbot_domain::external::set_qa_status_for_pull_requests;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::errors::DomainSnafu;
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
    let target_account = extract_account_from_auth(&mut *ctx.db_adapter.external_accounts(), &auth)
        .await
        .map_err(actix_web::Error::from)?;

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(target_account.username().into()),
            ..sentry::User::default()
        }));
    });

    // TODO: CAN EXPLODE
    let repo_path = RepositoryPath::from_str(&data.repository_path).unwrap();
    let status = match data.status {
        None => QaStatus::Waiting,
        Some(true) => QaStatus::Pass,
        Some(false) => QaStatus::Fail,
    };

    set_qa_status_for_pull_requests(
        &ctx.config,
        ctx.api_adapter.as_ref(),
        ctx.db_adapter.as_ref(),
        ctx.redis_adapter.as_ref(),
        &target_account,
        repo_path,
        &data.pull_request_numbers,
        &data.author,
        status,
    )
    .await
    .context(DomainSnafu)?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
