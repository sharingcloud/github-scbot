//! Pull requests logic.

use std::collections::HashMap;

use github_scbot_api::{
    checks::list_check_suites_for_git_ref, pulls::get_pull_request,
    reviews::list_reviews_for_pull_request,
};
use github_scbot_database::{
    models::{
        PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel, ReviewModel,
    },
    DbConn,
};
use github_scbot_types::{
    checks::GHCheckConclusion,
    common::GHUser,
    labels::StepLabel,
    pulls::{GHPullRequestAction, GHPullRequestEvent},
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
/// * `conn` - Database connection
/// * `event` - GitHub pull request event
pub async fn handle_pull_request_event(conn: &DbConn, event: &GHPullRequestEvent) -> Result<()> {
    let (repo_model, mut pr_model) =
        process_pull_request(conn, &event.repository, &event.pull_request)?;

    // Welcome message
    if let GHPullRequestAction::Opened = event.action {
        post_welcome_comment(&repo_model, &pr_model, &event.pull_request.user.login).await?;
    }

    let mut status_changed = false;

    // Status update
    match event.action {
        GHPullRequestAction::Opened | GHPullRequestAction::Synchronize => {
            pr_model.wip = event.pull_request.draft;
            pr_model.set_checks_status(CheckStatus::Waiting);
            pr_model.save(conn)?;
            status_changed = true;

            // Only for synchronize
            rerequest_existing_reviews(conn, &repo_model, &pr_model).await?;
        }
        GHPullRequestAction::Reopened | GHPullRequestAction::ReadyForReview => {
            pr_model.wip = event.pull_request.draft;
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ConvertedToDraft => {
            pr_model.wip = true;
            pr_model.save(conn)?;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequested => {
            handle_review_request(
                conn,
                &pr_model,
                GHReviewState::Pending,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )?;
            status_changed = true;
        }
        GHPullRequestAction::ReviewRequestRemoved => {
            handle_review_request(
                conn,
                &pr_model,
                GHReviewState::Dismissed,
                &extract_usernames(&event.pull_request.requested_reviewers),
            )?;
            status_changed = true;
        }
        _ => (),
    }

    if let GHPullRequestAction::Edited = event.action {
        // Update PR title
        pr_model.name = event.pull_request.title.clone();
        pr_model.save(conn)?;
        status_changed = true;
    }

    if status_changed {
        update_pull_request_status(
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
    } else {
        match pr_model.get_checks_status() {
            Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) | None => {
                if status.missing_required_reviews() {
                    StepLabel::AwaitingRequiredReview
                } else if status.missing_reviews() {
                    StepLabel::AwaitingReview
                } else {
                    match status.qa_status {
                        Some(QAStatus::Fail) => StepLabel::AwaitingChanges,
                        Some(QAStatus::Waiting) | None => StepLabel::AwaitingQA,
                        Some(QAStatus::Pass) | Some(QAStatus::Skipped) => {
                            if status.locked {
                                StepLabel::Locked
                            } else {
                                StepLabel::AwaitingMerge
                            }
                        }
                    }
                }
            }
            Some(CheckStatus::Waiting) => StepLabel::AwaitingChecks,
            Some(CheckStatus::Fail) => StepLabel::AwaitingChanges,
        }
    })
}

/// Synchronize pull request from upstream.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
pub async fn synchronize_pull_request(
    conn: &DbConn,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<PullRequestModel> {
    // Get upstream pull request
    let upstream_pr = get_pull_request(repository_owner, repository_name, pr_number).await?;
    // Get reviews
    let reviews =
        list_reviews_for_pull_request(repository_owner, repository_name, pr_number).await?;
    // Get upstream checks
    let check_suites =
        list_check_suites_for_git_ref(repository_owner, repository_name, &upstream_pr.head.sha)
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
            ..Default::default()
        },
    )?;

    let mut pr = PullRequestModel::get_or_create(
        conn,
        PullRequestCreation {
            repository_id: repo.id,
            name: upstream_pr.title.clone(),
            number: pr_number as i32,
            ..Default::default()
        },
    )?;

    // Update reviews
    let review_map: HashMap<&str, &GHReview> =
        reviews.iter().map(|r| (&r.user.login[..], r)).collect();
    for review in &reviews {
        ReviewModel::create_or_update(conn, pr.id, review.state, &review.user.login)?;
    }

    // Remove missing reviews
    let existing_reviews = pr.get_reviews(conn)?;
    for review in &existing_reviews {
        if !review_map.contains_key(&review.username[..]) {
            review.remove(conn)?;
        }
    }

    // Update PR
    pr.name = upstream_pr.title;
    pr.wip = upstream_pr.draft;
    pr.set_checks_status(status);

    // Remove label is PR is merged
    if upstream_pr.merged_at.is_some() {
        pr.remove_step_label();
        pr.merged = true;
    } else {
        // Determine step label
        let existing_reviews = pr.get_reviews(conn)?;
        let label = determine_automatic_step(&repo, &pr, &existing_reviews)?;
        pr.set_step_label(label);
        pr.merged = false;
    }
    pr.save(conn)?;
    Ok(pr)
}

fn extract_usernames(users: &[GHUser]) -> Vec<&str> {
    users.iter().map(|r| &r.login[..]).collect()
}
