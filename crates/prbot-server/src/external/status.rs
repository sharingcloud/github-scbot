//! External status handlers.

use std::str::FromStr;

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use prbot_core::use_cases::status::SetPullRequestQaStatusInterface;
use prbot_models::{QaStatus, RepositoryPath};
use prbot_sentry::sentry;
use serde::{Deserialize, Serialize};
use shaku::HasComponent;

use crate::{external::validator::extract_account_from_auth, server::AppContext};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QaStatusJson {
    repository_path: String,
    pull_request_numbers: Vec<u64>,
    author: String,
    status: Option<bool>,
}

#[tracing::instrument(skip_all, fields(
    repository_path = data.repository_path,
    pull_request_numbers = ?data.pull_request_numbers,
    author = data.author,
    status = data.status
), ret)]
pub(crate) async fn set_qa_status(
    ctx: web::Data<AppContext>,
    data: web::Json<QaStatusJson>,
    auth: BearerAuth,
) -> Result<HttpResponse> {
    let target_account = extract_account_from_auth(ctx.db_service.as_ref(), &auth)
        .await
        .map_err(actix_web::Error::from)?;

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(target_account.username.clone()),
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

    tokio::spawn(async move {
        sentry::configure_scope(|scope| {
            scope.set_user(Some(sentry::User {
                username: Some(target_account.username.clone()),
                ..sentry::User::default()
            }));
        });

        let set_pull_request_qa_status: &dyn SetPullRequestQaStatusInterface =
            ctx.core_module.resolve_ref();
        set_pull_request_qa_status
            .run(
                &ctx.as_core_context(),
                &target_account,
                repo_path,
                &data.pull_request_numbers,
                &data.author,
                status,
            )
            .await
            .unwrap()
    });

    Ok(HttpResponse::Accepted().body("Set QA status."))
}
