//! Admin module.

use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use percent_encoding::{percent_decode, percent_decode_str};
use prbot_models::{ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, PullRequestRule, Repository};
use serde::Serialize;

use crate::server::AppContext;

pub mod validator;

#[derive(Serialize)]
struct ExtendedRepository {
    repository: Repository,
    pull_requests: Vec<PullRequest>,
    merge_rules: Vec<MergeRule>,
    pull_request_rules: Vec<PullRequestRule>
}

#[derive(Serialize)]
struct ExtendedExternalAccount {
    external_account: ExternalAccount,
    rights: Vec<ExternalAccountRight>
}


#[tracing::instrument(skip_all)]
pub(crate) async fn repositories_list(
    ctx: web::Data<AppContext>,
    _auth: BearerAuth,
) -> Result<HttpResponse> {
    let mut output = vec![];
    let repositories = ctx.db_service.repositories_all().await.unwrap();

    for repository in repositories.into_iter() {
        let pull_requests = ctx.db_service.pull_requests_list(&repository.owner, &repository.name).await.unwrap();
        let merge_rules = ctx.db_service.merge_rules_list(&repository.owner, &repository.name).await.unwrap();
        let pull_request_rules = ctx.db_service.pull_request_rules_list(&repository.owner, &repository.name).await.unwrap();

        output.push(ExtendedRepository {
            repository,
            pull_requests,
            merge_rules,
            pull_request_rules
        })
    }

    Ok(HttpResponse::Ok().json(&output))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn accounts_list(
    ctx: web::Data<AppContext>,
    _auth: BearerAuth,
) -> Result<HttpResponse> {
    let accounts = ctx.db_service.accounts_all().await.unwrap();
    Ok(HttpResponse::Ok().json(&accounts))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn external_accounts_list(
    ctx: web::Data<AppContext>,
    _auth: BearerAuth,
) -> Result<HttpResponse> {
    let mut output = vec![];
    let external_accounts = ctx.db_service.external_accounts_all().await.unwrap();
    for external_account in external_accounts.into_iter() {
        let rights = ctx.db_service.external_account_rights_list(&external_account.username).await.unwrap();
        output.push(ExtendedExternalAccount {
            external_account,
            rights
        });
    }

    Ok(HttpResponse::Ok().json(&output))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn pull_request_rules_create(
    ctx: web::Data<AppContext>,
    rule: web::Json<PullRequestRule>,
    _auth: BearerAuth,
) -> Result<HttpResponse> {
    let output = ctx.db_service.pull_request_rules_create(rule.0).await.unwrap();
    Ok(HttpResponse::Ok().json(&output))
}

#[tracing::instrument(skip_all)]
pub(crate) async fn pull_request_rules_delete(
    ctx: web::Data<AppContext>,
    path: web::Path<(u64, String)>,
    _auth: BearerAuth,
) -> Result<HttpResponse> {
    let repository_id = &path.0;
    let repository = ctx.db_service.repositories_get_from_id_expect(*repository_id).await.unwrap();

    let rule_name = &path.1;
    let rule_name = percent_decode_str(rule_name).decode_utf8_lossy().to_string();
    ctx.db_service.pull_request_rules_delete(&repository.owner, &repository.name, &rule_name).await.unwrap();

    Ok(HttpResponse::NoContent().finish())
}
