//! Pull requests logic.

use github_scbot_core::config::Config;
use github_scbot_core::types::{
    labels::StepLabel,
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent},
};
use github_scbot_database::{DbServiceAll, PullRequest, Repository};
use github_scbot_ghapi::{adapter::ApiService, comments::CommentApi, labels::LabelApi};
use github_scbot_redis::RedisService;

use crate::{
    commands::CommandContext,
    use_cases::{
        pulls::DeterminePullRequestMergeStrategyUseCase, status::UpdatePullRequestStatusUseCase,
    },
};
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
    db_adapter: &mut dyn DbServiceAll,
    redis_adapter: &dyn RedisService,
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
                    PullRequestLogic::post_welcome_comment(
                        api_adapter,
                        repo_owner,
                        repo_name,
                        pr_number,
                        &event.pull_request.user.login,
                    )
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
    db_adapter: &mut dyn DbServiceAll,
    redis_adapter: &dyn RedisService,
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
        db_adapter: &mut dyn DbServiceAll,
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

    pub async fn synchronize_pull_request(
        config: &Config,
        db_adapter: &mut dyn DbServiceAll,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
    ) -> Result<()> {
        let repo =
            Self::get_or_create_repository(config, db_adapter, repository_owner, repository_name)
                .await?;

        if db_adapter
            .pull_requests_get(repository_owner, repository_name, pr_number)
            .await?
            .is_none()
        {
            db_adapter
                .pull_requests_create(
                    PullRequest {
                        number: pr_number,
                        ..Default::default()
                    }
                    .with_repository(&repo),
                )
                .await?;
        }

        Ok(())
    }

    /// Get merge commit title.
    pub fn get_merge_commit_title(upstream_pr: &GhPullRequest) -> String {
        format!("{} (#{})", upstream_pr.title, upstream_pr.number)
    }

    /// Try automerge pull request.
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %repo_owner,
            repo_name = %repo_name,
            pr_number = pr_number
        ),
        ret
    )]
    pub async fn try_automerge_pull_request(
        api_adapter: &dyn ApiService,
        db_adapter: &mut dyn DbServiceAll,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<bool> {
        let repository = db_adapter
            .repositories_get(repo_owner, repo_name)
            .await?
            .unwrap();
        let pull_request = db_adapter
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
            .unwrap();

        let commit_title = Self::get_merge_commit_title(upstream_pr);
        let strategy = if let Some(s) = pull_request.strategy_override {
            s
        } else {
            DeterminePullRequestMergeStrategyUseCase {
                db_service: db_adapter,
                repo_owner,
                repo_name,
                head_branch: &upstream_pr.base.reference,
                base_branch: &upstream_pr.head.reference,
                default_strategy: repository.default_strategy,
            }
            .run()
            .await?
        };

        if let Err(e) = api_adapter
            .pulls_merge(
                repo_owner,
                repo_name,
                pr_number,
                &commit_title,
                "",
                strategy,
            )
            .await
        {
            CommentApi::post_comment(
                api_adapter,
                repo_owner,
                repo_name,
                pr_number,
                &format!(
                    "Could not auto-merge this pull request: _{}_\nAuto-merge disabled",
                    e
                ),
            )
            .await?;
            Ok(false)
        } else {
            CommentApi::post_comment(
                api_adapter,
                repo_owner,
                repo_name,
                pr_number,
                &format!(
                    "Pull request successfully auto-merged! (strategy: '{}')",
                    strategy
                ),
            )
            .await?;
            Ok(true)
        }
    }

    /// Apply pull request step.
    #[tracing::instrument(skip(api_adapter))]
    pub async fn apply_pull_request_step(
        api_adapter: &dyn ApiService,
        owner: &str,
        name: &str,
        number: u64,
        step: Option<StepLabel>,
    ) -> Result<()> {
        LabelApi::set_step_label(api_adapter, owner, name, number, step)
            .await
            .map_err(Into::into)
    }

    /// Post welcome comment on a pull request.
    pub async fn post_welcome_comment(
        api_adapter: &dyn ApiService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        pr_author: &str,
    ) -> Result<()> {
        CommentApi::post_comment(
            api_adapter,
            repo_owner,
            repo_name,
            pr_number,
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                pr_author
            ),
        )
        .await?;

        Ok(())
    }
}
