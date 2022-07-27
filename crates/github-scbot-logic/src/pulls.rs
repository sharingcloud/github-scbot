//! Pull requests logic.

use std::collections::{hash_map::Entry, HashMap};

use github_scbot_core::config::Config;
use github_scbot_core::types::{
    checks::{GhCheckConclusion, GhCheckSuite},
    labels::StepLabel,
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent},
    status::CheckStatus,
};
use github_scbot_database2::{DbService, PullRequest, Repository};
use github_scbot_ghapi::{adapter::ApiService, comments::CommentApi, labels::LabelApi};
use github_scbot_redis::RedisService;

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
    db_adapter: &dyn DbService,
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
        .pull_requests()
        .get(repo_owner, repo_name, pr_number)
        .await?
    {
        Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
        None => {
            if PullRequestLogic::should_create_pull_request(config, &repo_model, &event) {
                let pr = PullRequest::builder()
                    .with_repository(&repo_model)
                    .number(event.pull_request.number)
                    .build()
                    .unwrap();
                let pr_model = db_adapter.pull_requests().create(pr).await?;

                // Get upstream pull request
                let upstream_pr = api_adapter
                    .pulls_get(repo_model.owner(), repo_model.name(), pr_model.number())
                    .await?;

                StatusLogic::update_pull_request_status(
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    repo_owner,
                    repo_name,
                    pr_number,
                    &upstream_pr,
                )
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
                CommandExecutor::execute_commands(
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    repo_owner,
                    repo_name,
                    pr_number,
                    &upstream_pr,
                    0,
                    &event.pull_request.user.login,
                    commands,
                )
                .await?;

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
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhPullRequestEvent,
) -> Result<()> {
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;

    let pr_model = match db_adapter
        .pull_requests()
        .get(repo_owner, repo_name, event.pull_request.number)
        .await?
    {
        Some(pr) => pr,
        None => return Ok(()),
    };

    let pr_number = pr_model.number();
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

        StatusLogic::update_pull_request_status(
            api_adapter,
            db_adapter,
            redis_adapter,
            repo_owner,
            repo_name,
            pr_number,
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
    #[tracing::instrument(skip(api_adapter), ret)]
    pub async fn get_checks_status_from_github(
        api_adapter: &dyn ApiService,
        repository_owner: &str,
        repository_name: &str,
        sha: &str,
        wait_for_initial_checks: bool,
        exclude_check_suite_ids: &[u64],
    ) -> Result<CheckStatus> {
        // Get upstream checks
        let check_suites = api_adapter
            .check_suites_list(repository_owner, repository_name, sha)
            .await?;

        // Extract status
        if check_suites.is_empty() {
            if wait_for_initial_checks {
                Ok(CheckStatus::Waiting)
            } else {
                Ok(CheckStatus::Skipped)
            }
        } else {
            let filtered = Self::filter_and_merge_check_suites(
                check_suites,
                wait_for_initial_checks,
                exclude_check_suite_ids,
            );

            Ok(filtered)
        }
    }

    /// Get or create repository.
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
        wait_for_initial_checks: bool,
        exclude_ids: &[u64],
    ) -> CheckStatus {
        let filtered = Self::filter_last_check_suites(check_suites, exclude_ids);
        Self::merge_check_suite_statuses(&filtered, wait_for_initial_checks)
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
    fn merge_check_suite_statuses(
        check_suites: &[GhCheckSuite],
        wait_for_initial_checks: bool,
    ) -> CheckStatus {
        let initial = if wait_for_initial_checks {
            CheckStatus::Waiting
        } else {
            CheckStatus::Skipped
        };

        check_suites
            .iter()
            .fold(None, |acc, s| match (&acc, &s.conclusion) {
                // Already failed, or current check suite is failing
                (Some(CheckStatus::Fail), _) | (_, Some(GhCheckConclusion::Failure)) => {
                    Some(CheckStatus::Fail)
                }
                // No status or checks already pass, and current check suite pass
                (None | Some(CheckStatus::Pass), Some(GhCheckConclusion::Success)) => {
                    Some(CheckStatus::Pass)
                }
                // No conclusion for current check suite
                (_, None) => Some(CheckStatus::Waiting),
                // Keep same status
                (_, _) => acc,
            })
            .unwrap_or(initial)
    }

    /// Ensure repository & pull request creation.
    #[tracing::instrument(skip(config, db_adapter))]
    pub async fn synchronize_pull_request(
        config: &Config,
        db_adapter: &dyn DbService,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
    ) -> Result<()> {
        let repo =
            Self::get_or_create_repository(config, db_adapter, repository_owner, repository_name)
                .await?;

        if db_adapter
            .pull_requests()
            .get(repository_owner, repository_name, pr_number)
            .await?
            .is_none()
        {
            db_adapter
                .pull_requests()
                .create(
                    PullRequest::builder()
                        .with_repository(&repo)
                        .number(pr_number)
                        .build()
                        .unwrap(),
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
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<bool> {
        let repository = db_adapter
            .repositories()
            .get(repo_owner, repo_name)
            .await?
            .unwrap();
        let pull_request = db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
            .await?
            .unwrap();

        let commit_title = Self::get_merge_commit_title(upstream_pr);
        let strategy = if let Some(s) = pull_request.strategy_override() {
            *s
        } else {
            PullRequestStatus::get_strategy_from_branches(
                db_adapter,
                repo_owner,
                repo_name,
                &upstream_pr.base.reference,
                &upstream_pr.head.reference,
                repository.default_strategy(),
            )
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

#[cfg(test)]
mod tests {
    use github_scbot_core::{
        time::{Duration, OffsetDateTime},
        types::{
            checks::{GhCheckConclusion, GhCheckStatus, GhCheckSuite},
            common::GhApplication,
            status::CheckStatus,
        },
    };

    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    pub fn test_merge_check_suite_statuses() {
        // No check suite, no need to wait
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[], false),
            CheckStatus::Skipped
        );

        // No check suite, but with initial checks wait
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(&[], true),
            CheckStatus::Waiting
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
                false
            ),
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
                false,
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
                false,
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
                false
            ),
            CheckStatus::Pass
        );

        // Success with skipped
        assert_eq!(
            PullRequestLogic::merge_check_suite_statuses(
                &[
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
                ],
                false
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
                false
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
                        ..base_suite.clone()
                    }
                ],
                false
            ),
            CheckStatus::Fail
        );

        // Two GitHub actions at different moments
        let now = OffsetDateTime::now_utc();
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
                false,
                &[]
            ),
            CheckStatus::Pass
        );
    }
}
