//! External status handlers.

use std::{str::FromStr, sync::Arc};

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use github_scbot_domain::use_cases::{
    checks::DetermineChecksStatusUseCase,
    pulls::{AutomergePullRequestUseCase, MergePullRequestUseCase, SetStepLabelUseCase},
    status::{
        BuildPullRequestStatusUseCase, SetPullRequestQaStatusUseCase,
        UpdatePullRequestStatusUseCase,
    },
    summary::PostSummaryCommentUseCase,
};
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

#[tracing::instrument(skip_all, fields(
    repository_path = data.repository_path,
    pull_request_numbers = ?data.pull_request_numbers,
    author = data.author,
    status = data.status
), ret)]
pub(crate) async fn set_qa_status(
    ctx: web::Data<Arc<AppContext>>,
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

    SetPullRequestQaStatusUseCase {
        config: &ctx.config,
        api_service: ctx.api_service.as_ref(),
        db_service: ctx.db_service.as_ref(),
        lock_service: ctx.lock_service.as_ref(),
        update_pull_request_status: &UpdatePullRequestStatusUseCase {
            api_service: ctx.api_service.as_ref(),
            db_service: ctx.db_service.as_ref(),
            lock_service: ctx.lock_service.as_ref(),
            set_step_label: &SetStepLabelUseCase {
                api_service: ctx.api_service.as_ref(),
            },
            automerge_pull_request: &AutomergePullRequestUseCase {
                db_service: ctx.db_service.as_ref(),
                api_service: ctx.api_service.as_ref(),
                merge_pull_request: &MergePullRequestUseCase {
                    api_service: ctx.api_service.as_ref(),
                },
            },
            post_summary_comment: &PostSummaryCommentUseCase {
                api_service: ctx.api_service.as_ref(),
                db_service: ctx.db_service.as_ref(),
                lock_service: ctx.lock_service.as_ref(),
            },
            build_pull_request_status: &BuildPullRequestStatusUseCase {
                api_service: ctx.api_service.as_ref(),
                db_service: ctx.db_service.as_ref(),
                determine_checks_status: &DetermineChecksStatusUseCase {
                    api_service: ctx.api_service.as_ref(),
                },
            },
        },
    }
    .run(
        &target_account,
        repo_path,
        &data.pull_request_numbers,
        &data.author,
        status,
    )
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;

    Ok(HttpResponse::Ok().body("Set QA status."))
}
