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
    models::{
        HistoryWebhookModel, MergeRuleModel, PullRequestCreation, PullRequestModel,
        RepositoryCreation, RepositoryModel, ReviewModel,
    },
    DbConn,
};
use github_scbot_types::{
    checks::GHCheckConclusion,
    common::GHUser,
    events::EventType,
    labels::StepLabel,
    pulls::{GHMergeStrategy, GHPullRequestAction, GHPullRequestEvent},
    reviews::{GHReview, GHReviewState},
    status::{CheckStatus, QAStatus},
};

use crate::{
    database::process_pull_request,
    reviews::{handle_review_request, rerequest_existing_reviews},
    status::{update_pull_request_status, PullRequestStatus},
    welcome::post_welcome_comment,
    Result,
};

/// Handle GitHub pull request event.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `event` - GitHub pull request event
pub async fn handle_pull_request_event(
    config: &Config,
    conn: &DbConn,
    event: &GHPullRequestEvent,
) -> Result<()> {
    let (repo_model, mut pr_model) =
        process_pull_request(config, conn, &event.repository, &event.pull_request)?;

    HistoryWebhookModel::create_for_now(
        conn,
        &repo_model,
        &pr_model,
        &event.sender.login,
        EventType::PullRequest,
        event,
    )?;

    // Welcome message
    if let GHPullRequestAction::Opened = event.action {
        // Define needed reviewers count.
        pr_model.needed_reviewers_count = repo_model.default_needed_reviewers_count;
        pr_model.save(&conn)?;

        post_welcome_comment(
            config,
            &repo_model,
            &pr_model,
            &event.pull_request.user.login,
        )
        .await?;
    }

    let mut status_changed = false;

    // Status update
    match event.action {
        GHPullRequestAction::Opened | GHPullRequestAction::Synchronize => {
            pr_model.set_from_upstream(&event.pull_request);
            pr_model.set_checks_status(CheckStatus::Waiting);
            pr_model.save(conn)?;
            status_changed = true;

            // Only for synchronize
            rerequest_existing_reviews(config, conn, &repo_model, &pr_model).await?;
        }
        GHPullRequestAction::Reopened | GHPullRequestAction::ReadyForReview => {
            pr_model.set_from_upstream(&event.pull_request);
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ConvertedToDraft => {
            pr_model.set_from_upstream(&event.pull_request);
            pr_model.wip = true;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequested => {
            handle_review_request(
                config,
                conn,
                &repo_model,
                &pr_model,
                GHReviewState::Pending,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )
            .await?;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequestRemoved => {
            handle_review_request(
                config,
                conn,
                &repo_model,
                &pr_model,
                GHReviewState::Dismissed,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )
            .await?;
            status_changed = true;
        }
        GHPullRequestAction::Closed => {
            pr_model.set_from_upstream(&event.pull_request);
            pr_model.save(conn)?;
            status_changed = true;
        }
        _ => (),
    }

    if let GHPullRequestAction::Edited = event.action {
        // Update PR title
        pr_model.set_from_upstream(&event.pull_request);
        pr_model.save(conn)?;
        status_changed = true;
    }

    if status_changed {
        update_pull_request_status(
            config,
            conn,
            &repo_model,
            &mut pr_model,
            &event.pull_request.head.sha,
        )
        .await?;
    }

    Ok(())
}

/// Determine automatic step for a pull request.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviews` - Reviews
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
        match pr_model.get_checks_status()? {
            CheckStatus::Pass | CheckStatus::Skipped => {
                if status.missing_required_reviews() {
                    StepLabel::AwaitingRequiredReview
                } else if status.missing_reviews() {
                    StepLabel::AwaitingReview
                } else {
                    match status.qa_status {
                        QAStatus::Fail => StepLabel::AwaitingChanges,
                        QAStatus::Waiting => StepLabel::AwaitingQA,
                        QAStatus::Pass | QAStatus::Skipped => {
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
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `base_branch` - Base branch
/// * `head_branch` - Head branch
pub fn get_merge_strategy_for_branches(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    base_branch: &str,
    head_branch: &str,
) -> GHMergeStrategy {
    match MergeRuleModel::get_from_branches(conn, repo_model.id, base_branch, head_branch)
        .map(|x| x.get_strategy())
    {
        Ok(e) => e,
        Err(_) => {
            match MergeRuleModel::get_from_branches(conn, repo_model.id, base_branch, "*")
                .map(|x| x.get_strategy())
            {
                Ok(e) => e,
                Err(_) => repo_model.get_default_merge_strategy(),
            }
        }
    }
}

/// Synchronize pull request from upstream.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
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
    let check_suites = list_check_suites_from_git_ref(
        config,
        repository_owner,
        repository_name,
        &upstream_pr.head.sha,
    )
    .await?;

    // Extract status
    let status: CheckStatus = {
        if check_suites.is_empty() {
            CheckStatus::Skipped
        } else {
            check_suites
                .iter()
                // Only fetch GitHub apps, like GitHub Actions
                .filter(|&s| s.app.owner.login == "github")
                .fold(CheckStatus::Pass, |acc, s| match (&acc, &s.conclusion) {
                    (CheckStatus::Fail, _) => CheckStatus::Fail,
                    (_, Some(GHCheckConclusion::Failure)) => CheckStatus::Fail,
                    (_, None) => CheckStatus::Waiting,
                    (_, _) => acc,
                })
        }
    };

    let repo = RepositoryModel::get_or_create(
        conn,
        RepositoryCreation {
            name: repository_name.into(),
            owner: repository_owner.into(),
            ..RepositoryCreation::default(config)
        },
    )?;

    let mut pr = PullRequestModel::get_or_create(
        conn,
        &repo,
        PullRequestCreation::from_upstream(&upstream_pr, &repo),
    )?;

    // Update reviews
    let review_map: HashMap<&str, &GHReview> =
        reviews.iter().map(|r| (&r.user.login[..], r)).collect();
    for review in &reviews {
        let permission = get_user_permission_on_repository(
            config,
            repository_owner,
            repository_name,
            &review.user.login,
        )
        .await?;
        ReviewModel::create_or_update(
            conn,
            &repo,
            &pr,
            &review.user.login,
            Some(review.state),
            None,
            Some(permission.can_write()),
        )?;
    }

    // Remove missing reviews
    let existing_reviews = pr.get_reviews(conn)?;
    for review in &existing_reviews {
        if !review_map.contains_key(&review.username[..]) {
            review.remove(conn)?;
        }
    }

    // Update PR
    pr.set_from_upstream(&upstream_pr);
    pr.set_checks_status(status);

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
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
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

fn extract_usernames(users: &[GHUser]) -> Vec<&str> {
    users.iter().map(|r| &r.login[..]).collect()
}
