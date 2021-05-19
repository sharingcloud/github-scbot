//! Pull requests logic.

use std::collections::HashMap;

use github_scbot_api::{
    auth::get_user_permission_on_repository,
    checks::list_check_suites_from_git_ref,
    comments::post_comment,
    pulls::{get_pull_request, merge_pull_request},
    reviews::list_reviews_for_pull_request,
};
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{
        HistoryWebhookModel, MergeRuleModel, PullRequestModel, RepositoryModel, ReviewModel,
        RuleBranch,
    },
    DatabaseError, DbConn, DbPool,
};
use github_scbot_types::{
    checks::GhCheckConclusion,
    common::GhUser,
    events::EventType,
    labels::StepLabel,
    pulls::{GhMergeStrategy, GhPullRequestAction, GhPullRequestEvent},
    reviews::{GhReview, GhReviewState},
    status::{CheckStatus, QaStatus},
};
use tracing::{debug, info};

use crate::{
    commands::{parse_commands, Command},
    reviews::{handle_review_request, rerequest_existing_reviews},
    status::{update_pull_request_status, PullRequestStatus},
    welcome::post_welcome_comment,
    Result,
};

/// Pull request opened status.
pub enum PullRequestOpenedStatus {
    /// Pull request is already created.
    AlreadyCreated,
    /// Pull request is created.
    Created,
    /// Pull request is ignored.
    Ignored,
}

pub(crate) fn should_create_pull_request(
    config: &Config,
    repo_model: &RepositoryModel,
    event: &GhPullRequestEvent,
) -> Result<bool> {
    if repo_model.manual_interaction {
        // Check for magic instruction to enable bot
        let commands = parse_commands(&config, &event.pull_request.body);
        for command in commands.into_iter().flatten() {
            if let Command::AdminEnable = command {
                return Ok(true);
            }
        }

        Ok(false)
    } else {
        Ok(true)
    }
}

/// Handle pull request Opened event.
pub async fn handle_pull_request_opened(
    config: Config,
    pool: DbPool,
    event: GhPullRequestEvent,
) -> Result<PullRequestOpenedStatus> {
    let conn = get_connection(&pool)?;

    // Get or create repository
    let repo_model =
        RepositoryModel::builder_from_github(&config, &event.repository).create_or_update(&conn)?;

    if let Err(DatabaseError::UnknownPullRequest(_, _)) =
        PullRequestModel::get_from_repository_and_number(
            &conn,
            &repo_model,
            event.pull_request.number,
        )
    {
        if should_create_pull_request(&config, &repo_model, &event)? {
            let (_, mut pr_model) = PullRequestModel::create_or_update_from_github(
                config.clone(),
                pool.clone(),
                event.repository.clone(),
                event.pull_request.clone(),
            )
            .await?;

            if config.server_enable_history_tracking {
                HistoryWebhookModel::builder(&repo_model, &pr_model)
                    .username(&event.sender.login)
                    .event_key(EventType::PullRequest)
                    .payload(&event)
                    .create(&conn)?;
            }

            pr_model.needed_reviewers_count = repo_model.default_needed_reviewers_count;
            pr_model.set_checks_status(CheckStatus::Waiting);
            pr_model.save(&conn)?;

            info!(
                repository_path = %repo_model.get_path(),
                pr_model = ?pr_model,
                message = "Creating pull request",
            );

            update_pull_request_status(
                &config,
                pool,
                &repo_model,
                &mut pr_model,
                &event.pull_request.head.sha,
            )
            .await?;

            post_welcome_comment(
                &config,
                &repo_model,
                &pr_model,
                &event.pull_request.user.login,
            )
            .await?;

            Ok(PullRequestOpenedStatus::Created)
        } else {
            Ok(PullRequestOpenedStatus::Ignored)
        }
    } else {
        Ok(PullRequestOpenedStatus::AlreadyCreated)
    }
}

/// Handle GitHub pull request event.
pub async fn handle_pull_request_event(
    config: Config,
    pool: DbPool,
    event: GhPullRequestEvent,
) -> Result<()> {
    let conn = get_connection(&pool)?;

    // Get or create repository
    let repo_model =
        RepositoryModel::builder_from_github(&config, &event.repository).create_or_update(&conn)?;

    if let Ok(pr_model) = PullRequestModel::get_from_repository_and_number(
        &conn,
        &repo_model,
        event.pull_request.number,
    ) {
        if config.server_enable_history_tracking {
            HistoryWebhookModel::builder(&repo_model, &pr_model)
                .username(&event.sender.login)
                .event_key(EventType::PullRequest)
                .payload(&event)
                .create(&conn)?;
        }

        // Update from GitHub
        let (repo_model, mut pr_model) = PullRequestModel::create_or_update_from_github(
            config.clone(),
            pool.clone(),
            event.repository.clone(),
            event.pull_request.clone(),
        )
        .await?;

        let mut status_changed = false;

        // Status update
        match event.action {
            GhPullRequestAction::Synchronize => {
                // Force status to waiting
                pr_model.set_checks_status(CheckStatus::Waiting);
                pr_model.save(&conn)?;
                status_changed = true;

                rerequest_existing_reviews(&config, &conn, &repo_model, &pr_model).await?;
            }
            GhPullRequestAction::Reopened | GhPullRequestAction::ReadyForReview => {
                status_changed = true;
            }
            GhPullRequestAction::ConvertedToDraft => {
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequested => {
                handle_review_request(
                    &config,
                    &conn,
                    &repo_model,
                    &pr_model,
                    GhReviewState::Pending,
                    &extract_usernames(&event.pull_request.requested_reviewers),
                )
                .await?;
                status_changed = true;
            }
            GhPullRequestAction::ReviewRequestRemoved => {
                handle_review_request(
                    &config,
                    &conn,
                    &repo_model,
                    &pr_model,
                    GhReviewState::Dismissed,
                    &extract_usernames(&event.pull_request.requested_reviewers),
                )
                .await?;
                status_changed = true;
            }
            GhPullRequestAction::Closed => {
                status_changed = true;
            }
            _ => (),
        }

        if let GhPullRequestAction::Edited = event.action {
            // Update PR title
            status_changed = true;
        }

        if status_changed {
            update_pull_request_status(
                &config,
                pool,
                &repo_model,
                &mut pr_model,
                &event.pull_request.head.sha,
            )
            .await?;
        }
    }

    Ok(())
}

/// Determine automatic step for a pull request.
pub fn determine_automatic_step(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
) -> Result<StepLabel> {
    let status = PullRequestStatus::from_pull_request(repo_model, pr_model, reviews)?;

    Ok(if pr_model.wip {
        StepLabel::Wip
    } else if !status.valid_pr_title {
        StepLabel::AwaitingChanges
    } else {
        match pr_model.get_checks_status() {
            CheckStatus::Pass | CheckStatus::Skipped => {
                if status.missing_required_reviews() {
                    StepLabel::AwaitingRequiredReview
                } else if status.missing_reviews() {
                    StepLabel::AwaitingReview
                } else {
                    match status.qa_status {
                        QaStatus::Fail => StepLabel::AwaitingChanges,
                        QaStatus::Waiting => StepLabel::AwaitingQa,
                        QaStatus::Pass | QaStatus::Skipped => {
                            if status.locked {
                                StepLabel::Locked
                            } else {
                                StepLabel::AwaitingMerge
                            }
                        }
                    }
                }
            }
            CheckStatus::Waiting => StepLabel::AwaitingChecks,
            CheckStatus::Fail => StepLabel::AwaitingChanges,
        }
    })
}

/// Get merge strategy for branches.
pub fn get_merge_strategy_for_branches(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    base_branch: &str,
    head_branch: &str,
) -> GhMergeStrategy {
    match MergeRuleModel::get_from_branches(conn, repo_model, base_branch, head_branch)
        .map(|x| x.get_strategy())
    {
        Ok(e) => e,
        Err(_) => {
            match MergeRuleModel::get_from_branches(
                conn,
                repo_model,
                base_branch,
                RuleBranch::Wildcard,
            )
            .map(|x| x.get_strategy())
            {
                Ok(e) => e,
                Err(_) => repo_model.get_default_merge_strategy(),
            }
        }
    }
}

/// Get checks status from GitHub.
pub async fn get_checks_status_from_github(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    sha: &str,
    exclude_check_suite_ids: &[u64],
) -> Result<CheckStatus> {
    // Get upstream checks
    let check_suites =
        list_check_suites_from_git_ref(config, repository_owner, repository_name, sha).await?;

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
        let filtered = check_suites
            .iter()
            // Only fetch GitHub Actions statuses
            .filter(|&s| {
                s.app.slug == "github-actions"
                    && !exclude_check_suite_ids.contains(&s.id)
                    && !s.pull_requests.is_empty()
            })
            .fold(CheckStatus::Pass, |acc, s| match (&acc, &s.conclusion) {
                (CheckStatus::Fail, _) => CheckStatus::Fail,
                (_, Some(GhCheckConclusion::Failure)) => CheckStatus::Fail,
                (_, None) => CheckStatus::Waiting,
                (_, _) => acc,
            });

        debug!(
            repository_path = %repository_path,
            sha = %sha,
            filtered = ?filtered,
            message = "Filtered check suites"
        );

        Ok(filtered)
    }
}

/// Synchronize pull request from upstream.
pub async fn synchronize_pull_request(
    config: &Config,
    conn: &DbConn,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<(PullRequestModel, String)> {
    // Get upstream pull request
    let upstream_pr =
        get_pull_request(config, repository_owner, repository_name, pr_number).await?;
    // Get reviews
    let reviews =
        list_reviews_for_pull_request(config, repository_owner, repository_name, pr_number).await?;
    // Get upstream checks
    let status = get_checks_status_from_github(
        config,
        repository_owner,
        repository_name,
        &upstream_pr.head.sha,
        &[],
    )
    .await?;

    let repo = RepositoryModel::builder(config, repository_owner, repository_name)
        .create_or_update(conn)?;

    let mut pr = PullRequestModel::builder_from_github(&repo, &upstream_pr)
        .check_status(status)
        .create_or_update(conn)?;

    // Update reviews
    let review_map: HashMap<&str, &GhReview> =
        reviews.iter().map(|r| (&r.user.login[..], r)).collect();
    for review in &reviews {
        let permission = get_user_permission_on_repository(
            config,
            repository_owner,
            repository_name,
            &review.user.login,
        )
        .await?;

        ReviewModel::builder(&repo, &pr, &review.user.login)
            .state(review.state)
            .valid(permission.can_write())
            .create_or_update(conn)?;
    }

    // Remove missing reviews
    let existing_reviews = pr.get_reviews(conn)?;
    for review in &existing_reviews {
        if !review_map.contains_key(&review.username[..]) {
            review.remove(conn)?;
        }
    }

    // Determine step label
    let existing_reviews = pr.get_reviews(conn)?;
    let label = determine_automatic_step(&repo, &pr, &existing_reviews)?;
    pr.set_step_label(label);

    if upstream_pr.merged_at.is_some() {
        pr.remove_step_label();
    }

    pr.save(conn)?;
    Ok((pr, upstream_pr.head.sha))
}

/// Try automerge pull request.
pub async fn try_automerge_pull_request(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<bool> {
    let commit_title = pr_model.get_merge_commit_title();
    let strategy = get_merge_strategy_for_branches(
        conn,
        repo_model,
        &pr_model.base_branch,
        &pr_model.head_branch,
    );

    match merge_pull_request(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &commit_title,
        "",
        strategy,
    )
    .await
    {
        Err(e) => {
            post_comment(
                config,
                &repo_model.owner,
                &repo_model.name,
                pr_model.get_number(),
                &format!(
                    "Could not auto-merge this pull request: _{}_\nAuto-merge disabled",
                    e
                ),
            )
            .await?;
            Ok(false)
        }
        _ => {
            post_comment(
                config,
                &repo_model.owner,
                &repo_model.name,
                pr_model.get_number(),
                &format!(
                    "Pull request successfully auto-merged! (strategy: '{}')",
                    strategy.to_string()
                ),
            )
            .await?;
            Ok(true)
        }
    }
}

fn extract_usernames(users: &[GhUser]) -> Vec<&str> {
    users.iter().map(|r| &r.login[..]).collect()
}
