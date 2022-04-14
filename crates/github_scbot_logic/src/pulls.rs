//! Pull requests logic.

use std::collections::{hash_map::Entry, HashMap};

use github_scbot_conf::Config;
use github_scbot_database2::{DbService, PullRequest, Repository};
use github_scbot_ghapi::{adapter::IAPIAdapter, comments::CommentApi, labels::LabelApi};
use github_scbot_redis::{IRedisAdapter, LockStatus};
use github_scbot_types::{
    checks::{GhCheckConclusion, GhCheckSuite},
    labels::StepLabel,
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent},
    status::CheckStatus,
};
use tracing::{debug, info};

use crate::{
    commands::{AdminCommand, Command, CommandExecutor, CommandParser},
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
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhPullRequestEvent,
) -> Result<PullRequestOpenedStatus> {
    // Get or create repository
    let owner = &event.repository.owner.login;
    let name = &event.repository.name;
    let number = event.pull_request.number;

    let repo_model =
        PullRequestLogic::get_or_create_repository(config, db_adapter, owner, name).await?;

    match db_adapter.pull_requests().get(owner, name, number).await? {
        Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
        None => {
            if PullRequestLogic::should_create_pull_request(config, &repo_model, &event) {
                let key = format!(
                    "pr-creation_{}-{}_{}",
                    repo_model.owner(),
                    repo_model.name(),
                    event.pull_request.number
                );
                if let LockStatus::SuccessfullyLocked(l) =
                    redis_adapter.try_lock_resource(&key).await?
                {
                    let pr = PullRequest::builder()
                        .from_repository(&repo_model)
                        .number(event.pull_request.number)
                        .build()
                        .unwrap();
                    let pr_model = db_adapter.pull_requests().create(pr).await?;

                    // Get upstream pull request
                    let upstream_pr = api_adapter
                        .pulls_get(repo_model.owner(), repo_model.name(), pr_model.number())
                        .await?;

                    let check_status = if repo_model.default_enable_checks() {
                        PullRequestLogic::get_checks_status_from_github(
                            api_adapter,
                            repo_model.owner(),
                            repo_model.name(),
                            &upstream_pr.head.sha,
                            &[],
                        )
                        .await?
                    } else {
                        CheckStatus::Skipped
                    };

                    info!(
                        repository_path = %repo_model.path(),
                        pr_model = ?pr_model,
                        check_status = ?check_status,
                        message = "Creating pull request",
                    );

                    StatusLogic::create_initial_pull_request_status(
                        api_adapter,
                        db_adapter,
                        &repo_model,
                        &pr_model,
                        &event.pull_request.head.sha,
                        &upstream_pr,
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
                        &repo_model,
                        &pr_model,
                        &upstream_pr,
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
        }
    }
}

/// Handle GitHub pull request event.
pub async fn handle_pull_request_event(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhPullRequestEvent,
) -> Result<()> {
    let owner = &event.repository.owner.login;
    let name = &event.repository.name;

    let pr_model = match db_adapter
        .pull_requests()
        .get(owner, name, event.pull_request.number)
        .await?
    {
        Some(pr) => pr,
        None => return Ok(()),
    };

    let repo_model = db_adapter.repositories().get(owner, name).await?.unwrap();
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
            .pulls_get(owner, name, pr_model.number())
            .await?;

        StatusLogic::update_pull_request_status(
            api_adapter,
            db_adapter,
            redis_adapter,
            &repo_model,
            &pr_model,
            &upstream_pr,
        )
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
    #[tracing::instrument(skip(api_adapter))]
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
            let filtered =
                Self::filter_and_merge_check_suites(check_suites, exclude_check_suite_ids);

            debug!(
                repository_path = %repository_path,
                sha = %sha,
                filtered = ?filtered,
                message = "Filtered check suites"
            );

            Ok(filtered)
        }
    }

    pub async fn get_or_create_repository(
        config: &Config,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
    ) -> Result<Repository> {
        Ok(
            match db_adapter.repositories().get(repo_owner, repo_name).await? {
                Some(r) => r,
                None => {
                    db_adapter
                        .repositories()
                        .create(
                            Repository::builder()
                                .owner(repo_owner)
                                .name(repo_name)
                                .with_config(config)
                                .build()
                                .unwrap(),
                        )
                        .await?
                }
            },
        )
    }

    /// Filter and merge check suites.
    pub fn filter_and_merge_check_suites(
        check_suites: Vec<GhCheckSuite>,
        exclude_ids: &[u64],
    ) -> CheckStatus {
        let filtered = Self::filter_last_check_suites(check_suites, exclude_ids);
        Self::merge_check_suite_statuses(&filtered)
    }

    /// Filter last check suites.
    fn filter_last_check_suites(
        check_suites: Vec<GhCheckSuite>,
        exclude_ids: &[u64],
    ) -> Vec<GhCheckSuite> {
        let mut map: HashMap<u64, GhCheckSuite> = HashMap::new();
        // Only fetch GitHub Actions statuses
        for check_suite in check_suites
            .into_iter()
            .filter(|s| s.app.slug == "github-actions" && !exclude_ids.contains(&s.id))
        {
            if let Entry::Vacant(e) = map.entry(check_suite.id) {
                e.insert(check_suite);
            } else {
                let entry = map.get_mut(&check_suite.id).unwrap();
                if entry.updated_at < check_suite.updated_at {
                    *entry = check_suite;
                }
            }
        }

        map.into_values().collect()
    }

    /// Merge check suite statuses.
    #[tracing::instrument]
    fn merge_check_suite_statuses(check_suites: &[GhCheckSuite]) -> CheckStatus {
        check_suites
            .iter()
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
    #[tracing::instrument(skip(config, api_adapter, db_adapter))]
    pub async fn synchronize_pull_request(
        config: &Config,
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn DbService,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
    ) -> Result<(PullRequest, String)> {
        // Get upstream pull request
        let upstream_pr = api_adapter
            .pulls_get(repository_owner, repository_name, pr_number)
            .await?;

        let repo =
            Self::get_or_create_repository(config, db_adapter, repository_owner, repository_name)
                .await?;
        let pr = match db_adapter
            .pull_requests()
            .get(repository_owner, repository_name, pr_number)
            .await?
        {
            Some(pr) => pr,
            None => {
                db_adapter
                    .pull_requests()
                    .create(
                        PullRequest::builder()
                            .from_repository(&repo)
                            .number(pr_number)
                            .build()
                            .unwrap(),
                    )
                    .await?
            }
        };

        Ok((pr, upstream_pr.head.sha))
    }

    pub fn get_merge_commit_title(upstream_pr: &GhPullRequest) -> String {
        format!("{} (#{})", upstream_pr.title, upstream_pr.number)
    }

    /// Try automerge pull request.
    #[tracing::instrument(skip(api_adapter, db_adapter))]
    pub async fn try_automerge_pull_request(
        api_adapter: &dyn IAPIAdapter,
        db_adapter: &dyn DbService,
        repo_model: &Repository,
        pr_model: &PullRequest,
        upstream_pr: &GhPullRequest,
    ) -> Result<bool> {
        let commit_title = Self::get_merge_commit_title(upstream_pr);
        let strategy = if let Some(s) = pr_model.strategy_override() {
            *s
        } else {
            PullRequestStatus::get_strategy_from_branches(
                db_adapter,
                repo_model.owner(),
                repo_model.name(),
                &upstream_pr.base.reference,
                &upstream_pr.head.reference,
                repo_model.default_strategy(),
            )
            .await?
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
    #[tracing::instrument(skip(api_adapter))]
    pub async fn apply_pull_request_step(
        api_adapter: &dyn IAPIAdapter,
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
        api_adapter: &dyn IAPIAdapter,
        repo_model: &Repository,
        pr_model: &PullRequest,
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
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
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
            PullRequestLogic::merge_check_suite_statuses(&[]),
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
            PullRequestLogic::merge_check_suite_statuses(&[GhCheckSuite {
                status: GhCheckStatus::Queued,
                conclusion: None,
                ..base_suite.clone()
            }]),
            CheckStatus::Waiting
        );

        // Suite should be skipped
        assert_eq!(
            PullRequestLogic::filter_and_merge_check_suites(
                vec![GhCheckSuite {
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
            PullRequestLogic::filter_and_merge_check_suites(
                vec![GhCheckSuite {
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
            PullRequestLogic::merge_check_suite_statuses(&[GhCheckSuite {
                status: GhCheckStatus::Completed,
                conclusion: Some(GhCheckConclusion::Success),
                ..base_suite.clone()
            }]),
            CheckStatus::Pass
        );

        // Success with skipped
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[
                GhCheckSuite {
                    id: 1,
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Success),
                    ..base_suite.clone()
                },
                GhCheckSuite {
                    id: 2,
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Skipped),
                    ..base_suite.clone()
                }
            ]),
            CheckStatus::Pass
        );

        // Success with queued
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[
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
            ]),
            CheckStatus::Waiting
        );

        // One failing check make the status fail
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[
                GhCheckSuite {
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Failure),
                    ..base_suite.clone()
                },
                GhCheckSuite {
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Success),
                    ..base_suite.clone()
                }
            ]),
            CheckStatus::Fail
        );

        // Two GitHub actions at different moments
        let now = Utc::now();
        assert_eq!(
            PullRequestLogic::filter_and_merge_check_suites(
                vec![
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        updated_at: now + Duration::hours(1),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        updated_at: now,
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 2,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_suite
                    }
                ],
                &[]
            ),
            CheckStatus::Pass
        );
    }
}
