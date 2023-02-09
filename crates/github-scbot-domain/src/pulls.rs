//! Pull requests logic.

use github_scbot_core::config::Config;
use github_scbot_core::types::pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent};
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequest, Repository};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use crate::use_cases::comments::PostWelcomeCommentUseCase;
use crate::{commands::CommandContext, use_cases::status::UpdatePullRequestStatusUseCase};
use crate::{
    commands::{AdminCommand, Command, CommandExecutor, CommandParser},
    Result,
};

/// Pull request opened status.
#[derive(Debug, PartialEq, Eq)]
pub enum PullRequestOpenedStatus {
    /// Pull request is already created.
    AlreadyCreated,
    /// Pull request is created.
    Created,
    /// Pull request is ignored.
    Ignored,
}

/// Handle pull request Opened event.
#[tracing::instrument(
    skip_all,
    fields(
        action = ?event.action,
        pr_number = event.number,
        repository_path = %event.repository.full_name,
        username = %event.pull_request.user.login
    )
)]
pub async fn handle_pull_request_opened(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhPullRequestEvent,
) -> Result<PullRequestOpenedStatus> {
    // Get or create repository
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;
    let pr_number = event.pull_request.number;

    let repo_model =
        PullRequestLogic::get_or_create_repository(config, db_adapter, repo_owner, repo_name)
            .await?;

    match db_adapter
        .pull_requests_get(repo_owner, repo_name, pr_number)
        .await?
    {
        Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
        None => {
            if PullRequestLogic::should_create_pull_request(config, &repo_model, &event) {
                let pr_model = db_adapter
                    .pull_requests_create(
                        PullRequest {
                            number: event.pull_request.number,
                            ..Default::default()
                        }
                        .with_repository(&repo_model),
                    )
                    .await?;

                // Get upstream pull request
                let upstream_pr = api_adapter
                    .pulls_get(&repo_model.owner, &repo_model.name, pr_model.number)
                    .await?;

                UpdatePullRequestStatusUseCase {
                    api_service: api_adapter,
                    db_service: db_adapter,
                    redis_service: redis_adapter,
                    repo_name,
                    repo_owner,
                    pr_number,
                    upstream_pr: &upstream_pr,
                }
                .run()
                .await?;

                if config.server_enable_welcome_comments {
                    PostWelcomeCommentUseCase {
                        api_service: api_adapter,
                        repo_owner,
                        repo_name,
                        pr_number,
                        pr_author: &event.pull_request.user.login,
                    }
                    .run()
                    .await?;
                }

                // Now, handle commands from body.
                let commands = CommandParser::parse_commands(
                    config,
                    &event.pull_request.body.unwrap_or_default(),
                );

                let mut ctx = CommandContext {
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    repo_owner,
                    repo_name,
                    pr_number,
                    upstream_pr: &upstream_pr,
                    comment_id: 0,
                    comment_author: &event.pull_request.user.login,
                };

                CommandExecutor::execute_commands(&mut ctx, commands).await?;

                Ok(PullRequestOpenedStatus::Created)
            } else {
                Ok(PullRequestOpenedStatus::Ignored)
            }
        }
    }
}

/// Handle GitHub pull request event.
#[tracing::instrument(
    skip_all,
    fields(
        action = ?event.action,
        pr_number = event.number,
        repository_path = %event.repository.full_name,
        username = %event.pull_request.user.login
    )
)]
pub async fn handle_pull_request_event(
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhPullRequestEvent,
) -> Result<()> {
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;

    let pr_model = match db_adapter
        .pull_requests_get(repo_owner, repo_name, event.pull_request.number)
        .await?
    {
        Some(pr) => pr,
        None => return Ok(()),
    };

    let pr_number = pr_model.number;
    let mut status_changed = false;

    // Status update
    match event.action {
        GhPullRequestAction::Synchronize => {
            // Force status to waiting
            status_changed = true;
        }
        GhPullRequestAction::Reopened
        | GhPullRequestAction::ReadyForReview
        | GhPullRequestAction::ConvertedToDraft
        | GhPullRequestAction::Closed => {
            status_changed = true;
        }
        GhPullRequestAction::ReviewRequested => {
            status_changed = true;
        }
        GhPullRequestAction::ReviewRequestRemoved => {
            status_changed = true;
        }
        _ => (),
    }

    if let GhPullRequestAction::Edited = event.action {
        // Update PR title
        status_changed = true;
    }

    if status_changed {
        let upstream_pr = api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?;

        UpdatePullRequestStatusUseCase {
            api_service: api_adapter,
            db_service: db_adapter,
            redis_service: redis_adapter,
            repo_name,
            repo_owner,
            pr_number,
            upstream_pr: &upstream_pr,
        }
        .run()
        .await?;
    }

    Ok(())
}

/// Pull request logic.
pub struct PullRequestLogic;

impl PullRequestLogic {
    pub(crate) fn should_create_pull_request(
        config: &Config,
        repo_model: &Repository,
        event: &GhPullRequestEvent,
    ) -> bool {
        if repo_model.manual_interaction {
            if let Some(body) = &event.pull_request.body {
                // Check for magic instruction to enable bot
                let commands = CommandParser::parse_commands(config, body);
                for command in commands.into_iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        return true;
                    }
                }
            }

            false
        } else {
            true
        }
    }

    /// Get or create repository.
    pub async fn get_or_create_repository(
        config: &Config,
        db_adapter: &mut dyn DbService,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<Repository> {
        Ok(
            match db_adapter.repositories_get(repo_owner, repo_name).await? {
                Some(r) => r,
                None => {
                    db_adapter
                        .repositories_create(
                            Repository {
                                owner: repo_owner.into(),
                                name: repo_name.into(),
                                ..Default::default()
                            }
                            .with_config(config),
                        )
                        .await?
                }
            },
        )
    }

    /// Get merge commit title.
    pub fn get_merge_commit_title(upstream_pr: &GhPullRequest) -> String {
        format!("{} (#{})", upstream_pr.title, upstream_pr.number)
    }
}
