//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain::{
    commands::CommandExecutor,
    use_cases::{
        checks::DetermineChecksStatusUseCase,
        comments::HandleIssueCommentEventUseCase,
        pulls::{
            AutomergePullRequestUseCase, MergePullRequestUseCase, SetStepLabelUseCase,
            SynchronizePullRequestUseCase,
        },
        status::{BuildPullRequestStatusUseCase, UpdatePullRequestStatusUseCase},
        summary::PostSummaryCommentUseCase,
    },
};
use github_scbot_ghapi_interface::{types::GhIssueCommentEvent, ApiService};
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{event_type::EventType, Result, ServerError};

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    config: &Config,
    api_service: &dyn ApiService,
    db_service: &dyn DbService,
    lock_service: &dyn LockService,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    let update_uc = UpdatePullRequestStatusUseCase {
        api_service,
        db_service,
        lock_service,
        set_step_label: &SetStepLabelUseCase { api_service },
        automerge_pull_request: &AutomergePullRequestUseCase {
            db_service,
            api_service,
            merge_pull_request: &MergePullRequestUseCase { api_service },
        },
        post_summary_comment: &PostSummaryCommentUseCase {
            api_service,
            db_service,
            lock_service,
        },
        build_pull_request_status: &BuildPullRequestStatusUseCase {
            api_service,
            db_service,
            determine_checks_status: &DetermineChecksStatusUseCase { api_service },
        },
    };

    HandleIssueCommentEventUseCase {
        config,
        api_service,
        db_service,
        lock_service,
        synchronize_pull_request: &SynchronizePullRequestUseCase { config, db_service },
        update_pull_request_status: &update_uc,
        command_executor: &CommandExecutor {
            db_service,
            update_pull_request_status: &update_uc,
        },
    }
    .run(event)
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
