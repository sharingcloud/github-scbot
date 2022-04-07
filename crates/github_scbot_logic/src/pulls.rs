//! Pull requests logic.

use github_scbot_conf::Config;
use github_scbot_database::{
    models::{
        HistoryWebhookModel, IDatabaseAdapter, MergeRuleModel, PullRequestModel, RepositoryModel,
    },
    DatabaseError,
};
use github_scbot_ghapi::{adapter::IAPIAdapter, comments::CommentApi, labels::LabelApi};
use github_scbot_redis::{IRedisAdapter, LockStatus};
use github_scbot_types::{
    checks::{GhCheckConclusion, GhCheckSuite},
    common::GhUser,
    events::EventType,
    pulls::{GhPullRequestAction, GhPullRequestEvent},
    reviews::GhReviewState,
    status::CheckStatus,
};
use tracing::{debug, info};

use crate::{
    commands::{AdminCommand, Command, CommandExecutor, CommandParser},
    reviews::ReviewLogic,
    status::{PullRequestStatus, StatusLogic},
    Result,
};

/// Pull request opened status.
#[derive(Debug, PartialEq)]
pub enum PullRequestOpenedStatus {
    /// Pull request is already created.
    AlreadyCreated,
    /// Pull request is created.
    Created,
    /// Pull request is ignored.
    Ignored,
}

/// Handle pull request Opened event.
pub async fn handle_pull_request_opened(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhPullRequestEvent,
) -> Result<PullRequestOpenedStatus> {
    // Get or create repository
    let mut repo_model = RepositoryModel::builder_from_github(config, &event.repository)
        .create_or_update(db_adapter.repository())
        .await?;

    if let Err(DatabaseError::UnknownPullRequest(_, _)) = db_adapter
        .pull_request()
        .get_from_repository_and_number(&repo_model, event.pull_request.number)
        .await
    {
        if PullRequestLogic::should_create_pull_request(config, &repo_model, &event) {
            let key = format!(
                "pr-creation_{}-{}_{}",
                repo_model.owner(),
                repo_model.name(),
                event.pull_request.number
            );
            if let LockStatus::SuccessfullyLocked(l) = redis_adapter.try_lock_resource(&key).await?
            {
                let (_, mut pr_model) = PullRequestModel::create_or_update_from_github(
                    config.clone(),
                    db_adapter.pull_request(),
                    db_adapter.repository(),
                    &event.repository,
                    &event.pull_request,
                )
                .await?;

                if config.server_enable_history_tracking {
                    HistoryWebhookModel::builder(&repo_model, &pr_model)
                        .username(&event.sender.login)
                        .event_key(EventType::PullRequest)
                        .payload(&event)
                        .create(db_adapter.history_webhook())
                        .await?;
                }

                let check_status = if repo_model.default_enable_checks() {
                    // Set waiting for now, but check GitHub just in case
                    CheckStatus::Waiting
                } else {
                    CheckStatus::Skipped
                };

                let update = pr_model
                    .create_update()
                    .check_status(check_status)
                    .needed_reviewers_count(repo_model.default_needed_reviewers_count() as u64)
                    .build_update();
                db_adapter
                    .pull_request()
                    .update(&mut pr_model, update)
                    .await?;

                info!(
                    repository_path = %repo_model.path(),
                    pr_model = ?pr_model,
                    message = "Creating pull request",
                );

                StatusLogic::create_initial_pull_request_status(
                    api_adapter,
                    db_adapter,
                    &repo_model,
                    &mut pr_model,
                    &event.pull_request.head.sha,
                )
                .await?;

                if config.server_enable_welcome_comments {
                    PullRequestLogic::post_welcome_comment(
                        api_adapter,
                        &repo_model,
                        &pr_model,
                        &event.pull_request.user.login,
                    )
                    .await?;
                }

                l.release().await?;

                // Now, handle commands from body.
                let commands = CommandParser::parse_commands(
                    config,
                    &event.pull_request.body.unwrap_or_default(),
                );
                CommandExecutor::execute_commands(
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    &mut repo_model,
                    &mut pr_model,
                    0,
                    &event.pull_request.user.login,
                    commands,
                )
                .await?;

                Ok(PullRequestOpenedStatus::Created)
            } else {
                Ok(PullRequestOpenedStatus::AlreadyCreated)
            }
        } else {
            Ok(PullRequestOpenedStatus::Ignored)
        }
    } else {
        Ok(PullRequestOpenedStatus::AlreadyCreated)
    }
}

/// Handle GitHub pull request event.
pub async fn handle_pull_request_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhPullRequestEvent,
) -> Result<()> {
    // Get or create repository
    let repo_model = RepositoryModel::builder_from_github(config, &event.repository)
        .create_or_update(db_adapter.repository())
        .await?;

    if let Ok(pr_model) = db_adapter
        .pull_request()
        .get_from_repository_and_number(&repo_model, event.pull_request.number)
        .await
    {
        if config.server_enable_history_tracking {
            HistoryWebhookModel::builder(&repo_model, &pr_model)
                .username(&event.sender.login)
                .event_key(EventType::PullRequest)
                .payload(&event)
                .create(db_adapter.history_webhook())
                .await?;
        }

        // Update from GitHub
        let (repo_model, mut pr_model) = PullRequestModel::create_or_update_from_github(
            config.clone(),
            db_adapter.pull_request(),
            db_adapter.repository(),
            &event.repository,
            &event.pull_request,
        )
        .await?;

        let mut status_changed = false;

        // Status update
        match event.action {
            GhPullRequestAction::Synchronize => {
                // Force status to waiting
                let update = pr_model
                    .create_update()
                    .check_status(CheckStatus::Waiting)
                    .build_update();
                db_adapter
                    .pull_request()
                    .update(&mut pr_model, update)
                    .await?;
                status_changed = true;

                ReviewLogic::rerequest_existing_reviews(
                    api_adapter,
                    db_adapter,
                    &repo_model,
                    &pr_model,
                )
                .await?;
            }
            GhPullRequestAction::Reopened
            | GhPullRequestAction::ReadyForReview
            | GhPullRequestAction::ConvertedToDraft
            | GhPullRequestAction::Closed => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequested => {
                ReviewLogic::process_review_request(
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    &repo_model,
                    &pr_model,
                    GhReviewState::Pending,
                    &PullRequestLogic::extract_usernames(&event.pull_request.requested_reviewers),
                )
                .await?;
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequestRemoved => {
                ReviewLogic::process_review_request(
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    &repo_model,
                    &pr_model,
                    GhReviewState::Dismissed,
                    &PullRequestLogic::extract_usernames(&event.pull_request.requested_reviewers),
                )
                .await?;
                status_changed = true;
            }
            _ => (),
        }

        if let GhPullRequestAction::Edited = event.action {
            // Update PR title
            status_changed = true;
        }

        if status_changed {
            StatusLogic::update_pull_request_status(
                api_adapter,
                db_adapter,
                redis_adapter,
                &repo_model,
                &mut pr_model,
                &event.pull_request.head.sha,
            )
            .await?;
        }
    }

    Ok(())
}

/// Pull request logic.
pub struct PullRequestLogic;

impl PullRequestLogic {
    pub(crate) fn should_create_pull_request(
        config: &Config,
        repo_model: &RepositoryModel,
        event: &GhPullRequestEvent,
    ) -> bool {
        if repo_model.manual_interaction() {
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

    /// Get checks status from GitHub.
    pub async fn get_checks_status_from_github(
        api_adapter: &dyn IAPIAdapter,
        repository_owner: &str,
        repository_name: &str,
        sha: &str,
        exclude_check_suite_ids: &[u64],
    ) -> Result<CheckStatus> {
        // Get upstream checks
        let check_suites = api_adapter
            .check_suites_list(repository_owner, repository_name, sha)
            .await?;

        let repository_path = format!("{}/{}", repository_owner, repository_name);

        debug!(
            repository_path = %repository_path,
            sha = %sha,
            check_suites = ?check_suites,
            message = "Check suites status from GitHub"
        );

        // Extract status
        if check_suites.is_empty() {
            debug!(
                repository_path = %repository_path,
                sha = %sha,
                message = "No check suite found from GitHub"
            );

            Ok(CheckStatus::Skipped)
        } else {
            let filtered = Self::merge_check_suite_statuses(&check_suites, exclude_check_suite_ids);

            debug!(
                repository_path = %repository_path,
                sha = %sha,
                filtered = ?filtered,
                message = "Filtered check suites"
            );

            Ok(filtered)
        }
    }

    /// Merge check suite statuses.
    pub fn merge_check_suite_statuses(
        check_suites: &[GhCheckSuite],
        exclude_ids: &[u64],
    ) -> CheckStatus {
        check_suites
            .iter()
            // Only fetch GitHub Actions statuses
            .filter(|&s| s.app.slug == "github-actions" && !exclude_ids.contains(&s.id))
            .fold(CheckStatus::Skipped, |acc, s| match (&acc, &s.conclusion) {
                (CheckStatus::Fail, _) | (_, Some(GhCheckConclusion::Failure)) => CheckStatus::Fail,
                (CheckStatus::Skipped | CheckStatus::Pass, Some(GhCheckConclusion::Success)) => {
                    CheckStatus::Pass
                }
                (_, None) => CheckStatus::Waiting,
                (_, _) => acc,
            })
    }

    /// Synchronize pull request from upstream.
    pub async fn synchronize_pull_request(
        config: &Config,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
    ) -> Result<(PullRequestModel, String)> {
        // Get upstream pull request
        let upstream_pr = api_adapter
            .pulls_get(repository_owner, repository_name, pr_number)
            .await?;

        // Get upstream checks
        let status = Self::get_checks_status_from_github(
            api_adapter,
            repository_owner,
            repository_name,
            &upstream_pr.head.sha,
            &[],
        )
        .await?;

        let repo = RepositoryModel::builder(config, repository_owner, repository_name)
            .create_or_update(db_adapter.repository())
            .await?;

        let mut pr = PullRequestModel::builder_from_github(&repo, &upstream_pr)
            .check_status(status)
            .create_or_update(db_adapter.pull_request())
            .await?;

        ReviewLogic::synchronize_reviews(api_adapter, db_adapter, &repo, &pr).await?;

        // Determine step label
        let pr_status =
            PullRequestStatus::from_database(api_adapter, db_adapter, &repo, &pr).await?;
        let label = StatusLogic::determine_automatic_step(&pr_status)?;
        let label = if upstream_pr.merged_at.is_some() {
            None
        } else {
            Some(label)
        };

        let update = pr.create_update().step(label).build_update();

        db_adapter.pull_request().update(&mut pr, update).await?;
        Ok((pr, upstream_pr.head.sha))
    }

    /// Try automerge pull request.
    pub async fn try_automerge_pull_request(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
    ) -> Result<bool> {
        let commit_title = pr_model.merge_commit_title();
        let strategy = if let Some(s) = pr_model.strategy_override() {
            s
        } else {
            MergeRuleModel::get_strategy_from_branches(
                db_adapter.merge_rule(),
                repo_model,
                pr_model.base_branch(),
                pr_model.head_branch(),
            )
            .await
        };

        if let Err(e) = api_adapter
            .pulls_merge(
                repo_model.owner(),
                repo_model.name(),
                pr_model.number(),
                &commit_title,
                "",
                strategy,
            )
            .await
        {
            CommentApi::post_comment(
                api_adapter,
                repo_model.owner(),
                repo_model.name(),
                pr_model.number(),
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
                repo_model.owner(),
                repo_model.name(),
                pr_model.number(),
                &format!(
                    "Pull request successfully auto-merged! (strategy: '{}')",
                    strategy.to_string()
                ),
            )
            .await?;
            Ok(true)
        }
    }

    /// Apply pull request step.
    pub async fn apply_pull_request_step(
        api_adapter: &dyn IAPIAdapter,
        repository_model: &RepositoryModel,
        pr_model: &PullRequestModel,
    ) -> Result<()> {
        LabelApi::set_step_label(
            api_adapter,
            repository_model.owner(),
            repository_model.name(),
            pr_model.number(),
            pr_model.step(),
        )
        .await
        .map_err(Into::into)
    }

    /// Post welcome comment on a pull request.
    pub async fn post_welcome_comment(
        api_adapter: &dyn IAPIAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        pr_author: &str,
    ) -> Result<()> {
        CommentApi::post_comment(
            api_adapter,
            repo_model.owner(),
            repo_model.name(),
            pr_model.number(),
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                pr_author
            ),
        )
        .await?;

        Ok(())
    }

    fn extract_usernames(users: &[GhUser]) -> Vec<&str> {
        users.iter().map(|r| &r.login[..]).collect()
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_types::{
        checks::{GhCheckConclusion, GhCheckStatus, GhCheckSuite},
        common::GhApplication,
        status::CheckStatus,
    };

    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    pub fn test_merge_check_suite_statuses() {
        // No check suite, no need to wait
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[], &[]),
            CheckStatus::Skipped
        );

        let base_suite = GhCheckSuite {
            app: GhApplication {
                slug: "github-actions".into(),
                ..GhApplication::default()
            },
            ..GhCheckSuite::default()
        };

        // Should wait on queued status
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[GhCheckSuite {
                    status: GhCheckStatus::Queued,
                    conclusion: None,
                    ..base_suite.clone()
                }],
                &[]
            ),
            CheckStatus::Waiting
        );

        // Suite should be skipped
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[GhCheckSuite {
                    id: 1,
                    status: GhCheckStatus::Queued,
                    conclusion: None,
                    ..base_suite.clone()
                }],
                &[1]
            ),
            CheckStatus::Skipped
        );

        // Ignore unsupported apps
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[GhCheckSuite {
                    status: GhCheckStatus::Queued,
                    app: GhApplication {
                        slug: "toto".into(),
                        ..GhApplication::default()
                    },
                    ..GhCheckSuite::default()
                }],
                &[]
            ),
            CheckStatus::Skipped
        );

        // Success
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[GhCheckSuite {
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Success),
                    ..base_suite.clone()
                }],
                &[]
            ),
            CheckStatus::Pass
        );

        // Success with skipped
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[
                    GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_suite.clone()
                    }
                ],
                &[]
            ),
            CheckStatus::Pass
        );

        // Success with queued
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[
                    GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        status: GhCheckStatus::Queued,
                        conclusion: None,
                        ..base_suite.clone()
                    }
                ],
                &[]
            ),
            CheckStatus::Waiting
        );

        // One failing check make the status fail
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[
                    GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite
                    }
                ],
                &[]
            ),
            CheckStatus::Fail
        );
    }
}
