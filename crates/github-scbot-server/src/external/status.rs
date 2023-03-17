//! External status handlers.

use std::{str::FromStr, sync::Arc};

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_domain::use_cases::status::SetPullRequestQaStatusUseCase;
use github_scbot_domain_models::{QaStatus, RepositoryPath};
use github_scbot_sentry::sentry;
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
    let target_account = extract_account_from_auth(ctx.db_service.lock().await.as_mut(), &auth)
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

    SetPullRequestQaStatusUseCase {
        config: &ctx.config,
        api_service: ctx.api_service.as_ref(),
        db_service: ctx.db_service.lock().await.as_mut(),
        lock_service: ctx.lock_service.as_ref(),
        external_account: &target_account,
        repository_path: repo_path,
        pull_request_numbers: &data.pull_request_numbers,
        author: &data.author,
        status,
    }
    .run()
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
