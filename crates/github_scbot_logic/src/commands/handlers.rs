use github_scbot_conf::Config;
use github_scbot_database2::{DbService, RequiredReviewer};
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;
use github_scbot_types::{
    issues::GhReactionType,
    labels::StepLabel,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::{CheckStatus, QaStatus},
};

use super::command::CommandExecutionResult;
use crate::{
    auth::AuthLogic,
    commands::command::ResultAction,
    errors::Result,
    gif::GifPoster,
    pulls::PullRequestLogic,
    status::{PullRequestStatus, StatusLogic},
    summary::SummaryCommentSender,
};

/// Handle `Automerge` command.
pub async fn handle_auto_merge_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_author: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    db_adapter
        .pull_requests()
        .set_automerge(repo_owner, repo_name, pr_number, status)
        .await?;

    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by **{}**", status_text, comment_author);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::PostComment(comment))
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `Merge` command.
#[allow(clippy::too_many_arguments)]
pub async fn handle_merge_command(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    upstream_pr: &GhPullRequest,
    comment_author: &str,
    merge_strategy: Option<GhMergeStrategy>,
) -> Result<CommandExecutionResult> {
    // Use step to determine merge possibility
    let pr_status = PullRequestStatus::from_database(
        api_adapter,
        db_adapter,
        repo_owner,
        repo_name,
        pr_number,
        upstream_pr,
    )
    .await?;
    let step = StatusLogic::determine_automatic_step(&pr_status);
    let commit_title = PullRequestLogic::get_merge_commit_title(upstream_pr);
    let mut actions = vec![];

    if step == StepLabel::AwaitingMerge {
        if let Err(e) = api_adapter
            .pulls_merge(
                repo_owner,
                repo_name,
                pr_number,
                &commit_title,
                "",
                merge_strategy.unwrap_or(pr_status.merge_strategy),
            )
            .await
        {
            actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
            actions.push(ResultAction::PostComment(format!(
                "Could not merge this pull request: _{}_",
                e
            )));
        } else {
            actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
            actions.push(ResultAction::PostComment(format!(
                "Pull request successfully merged by {}! (strategy: '{}')",
                comment_author, pr_status.merge_strategy
            )));
        }
    } else {
        actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
        actions.push(ResultAction::PostComment(
            "Pull request is not ready to merge.".into(),
        ));
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_actions(actions)
        .build())
}

/// Handle `IsAdmin` command.
pub async fn handle_is_admin_command(
    db_adapter: &dyn DbService,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let known_admins = AuthLogic::list_known_admin_usernames(db_adapter).await?;
    let status = AuthLogic::is_admin(comment_author, &known_admins);
    let reaction_type = if status {
        GhReactionType::PlusOne
    } else {
        GhReactionType::MinusOne
    };

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(reaction_type))
        .build())
}

/// Handle `AdminSync` command.
pub async fn handle_admin_sync_command(
    config: &Config,
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    number: u64,
) -> Result<CommandExecutionResult> {
    PullRequestLogic::synchronize_pull_request(config, db_adapter, repo_owner, repo_name, number)
        .await?;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

pub async fn handle_admin_reset_summary_command(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    upstream_pr: &GhPullRequest,
) -> Result<CommandExecutionResult> {
    let status = PullRequestStatus::from_database(
        api_adapter,
        db_adapter,
        repo_owner,
        repo_name,
        pr_number,
        upstream_pr,
    )
    .await?;

    // Reset comment ID
    db_adapter
        .pull_requests()
        .set_status_comment_id(repo_owner, repo_name, pr_number, 0)
        .await?;

    SummaryCommentSender::create_or_update(
        api_adapter,
        db_adapter,
        redis_adapter,
        repo_owner,
        repo_name,
        pr_number,
        &status,
    )
    .await?;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `SkipQA` command.
pub async fn handle_skip_qa_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_author: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    let qa_status = if status {
        QaStatus::Skipped
    } else {
        QaStatus::Waiting
    };

    db_adapter
        .pull_requests()
        .set_qa_status(repo_owner, repo_name, pr_number, qa_status)
        .await?;
    let comment = format!("QA is marked as skipped by **{}**.", comment_author);

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `SkipChecks` command.
pub async fn handle_skip_checks_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_author: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    let check_status = if status {
        CheckStatus::Skipped
    } else {
        CheckStatus::Waiting
    };

    db_adapter
        .pull_requests()
        .set_checks_enabled(repo_owner, repo_name, pr_number, status)
        .await?;

    let comment = format!(
        "Checks are marked as {:?} by **{}**.",
        check_status, comment_author
    );

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `QaStatus` command.
pub async fn handle_qa_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_author: &str,
    status: Option<bool>,
) -> Result<CommandExecutionResult> {
    let (status, status_text) = match status {
        Some(true) => (QaStatus::Pass, "marked as pass"),
        Some(false) => (QaStatus::Fail, "marked as fail"),
        None => (QaStatus::Waiting, "marked as waiting"),
    };

    db_adapter
        .pull_requests()
        .set_qa_status(repo_owner, repo_name, pr_number, status)
        .await?;

    let comment = format!("QA is {} by **{}**.", status_text, comment_author);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Ping` command.
pub fn handle_ping_command(comment_author: &str) -> Result<CommandExecutionResult> {
    let comment = format!("**{}** pong!", comment_author);
    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Gif` command.
pub async fn handle_gif_command(
    config: &Config,
    api_adapter: &dyn ApiService,
    search_terms: &str,
) -> Result<CommandExecutionResult> {
    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(
            GifPoster::generate_random_gif_comment(config, api_adapter, search_terms).await?,
        ))
        .build())
}

pub async fn handle_set_merge_strategy(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    strategy: GhMergeStrategy,
) -> Result<CommandExecutionResult> {
    db_adapter
        .pull_requests()
        .set_strategy_override(repo_owner, repo_name, pr_number, Some(strategy))
        .await?;

    let comment = format!(
        "Merge strategy override set to '{}' for this pull request.",
        strategy
    );
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_unset_merge_strategy(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
) -> Result<CommandExecutionResult> {
    db_adapter
        .pull_requests()
        .set_strategy_override(repo_owner, repo_name, pr_number, None)
        .await?;

    let comment = "Merge strategy override removed for this pull request.".into();
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_set_labels(
    api_adapter: &dyn ApiService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    labels: &[String],
) -> Result<CommandExecutionResult> {
    api_adapter
        .issue_labels_add(repo_owner, repo_name, pr_number, labels)
        .await?;

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

pub async fn handle_unset_labels(
    api_adapter: &dyn ApiService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    labels: &[String],
) -> Result<CommandExecutionResult> {
    api_adapter
        .issue_labels_remove(repo_owner, repo_name, pr_number, labels)
        .await?;

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `AssignRequiredReviewers` command.
pub async fn handle_assign_required_reviewers_command(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    assert!(!reviewers.is_empty());

    let pr_model = db_adapter
        .pull_requests()
        .get(repo_owner, repo_name, pr_number)
        .await?
        .unwrap();

    let mut approved_reviewers = vec![];
    let mut rejected_reviewers = vec![];

    for reviewer in &reviewers {
        let permission = api_adapter
            .user_permissions_get(repo_owner, repo_name, reviewer)
            .await?
            .can_write();

        if permission {
            approved_reviewers.push(reviewer.clone());

            match db_adapter
                .required_reviewers()
                .get(repo_owner, repo_name, pr_number, reviewer)
                .await?
            {
                Some(_s) => (),
                None => {
                    db_adapter
                        .required_reviewers()
                        .create(
                            RequiredReviewer::builder()
                                .with_pull_request(&pr_model)
                                .username(reviewer)
                                .build()
                                .unwrap(),
                        )
                        .await?;
                }
            }
        } else {
            rejected_reviewers.push(reviewer.clone());
        }
    }

    let approved_len = approved_reviewers.len();
    let rejected_len = rejected_reviewers.len();

    if approved_len > 0 {
        // Communicate to GitHub
        api_adapter
            .pull_reviewer_requests_add(repo_owner, repo_name, pr_number, &approved_reviewers)
            .await?;
    }

    let mut comment = String::new();

    match approved_len {
        0 => (),
        1 => comment.push_str(&format!(
            "**{}** is now a required reviewer on this PR.",
            approved_reviewers[0]
        )),
        _ => comment.push_str(&format!(
            "**{}** are now required reviewers on this PR.",
            approved_reviewers.join(", ")
        )),
    }

    if approved_len > 0 && rejected_len > 0 {
        comment.push_str("\n\nBut");
    }

    match rejected_len {
        0 => (),
        1 => comment.push_str(&format!(
            "**{}** has no write permission on this repository and can't be a required reviewer.",
            rejected_reviewers[0]
        )),
        _ => comment.push_str(&format!(
            "**{}** have no write permission on this repository and can't be required reviewers.",
            rejected_reviewers.join(", ")
        )),
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(approved_len > 0)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `UnassignRequiredReviewers` command.
pub async fn handle_unassign_required_reviewers_command(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    assert!(!reviewers.is_empty());

    api_adapter
        .pull_reviewer_requests_remove(repo_owner, repo_name, pr_number, &reviewers)
        .await?;

    for reviewer in &reviewers {
        db_adapter
            .required_reviewers()
            .delete(repo_owner, repo_name, pr_number, reviewer)
            .await?;
    }

    let comment = if reviewers.len() == 1 {
        format!(
            "{} is not anymore a required reviewer on this PR.",
            reviewers[0]
        )
    } else {
        format!(
            "{} are not anymore required reviewers on this PR.",
            reviewers.join(" ")
        )
    };

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Lock` command.
pub async fn handle_lock_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_author: &str,
    status: bool,
    reason: Option<String>,
) -> Result<CommandExecutionResult> {
    let status_text = if status { "locked" } else { "unlocked" };
    db_adapter
        .pull_requests()
        .set_locked(repo_owner, repo_name, pr_number, status)
        .await?;

    let mut comment = format!("Pull request {} by **{}**.", status_text, comment_author);
    if let Some(reason) = reason {
        comment = format!("{}\n**Reason**: {}.", comment, reason);
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle "Set default needed reviewers" command.
pub async fn handle_set_default_needed_reviewers_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    count: u64,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_default_needed_reviewers_count(repo_owner, repo_name, count)
        .await?;

    let comment = format!(
        "Needed reviewers count set to **{}** for this repository.",
        count
    );
    Ok(CommandExecutionResult::builder()
        .with_status_update(false)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle "Set default merge strategy" command.
pub async fn handle_set_default_merge_strategy_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    strategy: GhMergeStrategy,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_default_strategy(repo_owner, repo_name, strategy)
        .await?;

    let comment = format!(
        "Merge strategy set to **{}** for this repository.",
        strategy
    );
    Ok(CommandExecutionResult::builder()
        .with_status_update(false)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle "Set default PR title regex" command.
pub async fn handle_set_default_pr_title_regex_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_title_regex: String,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_pr_title_validation_regex(repo_owner, repo_name, &pr_title_regex)
        .await?;

    let comment = if pr_title_regex.is_empty() {
        "PR title regex unset for this repository.".into()
    } else {
        format!(
            "PR title regex set to **{}** for this repository.",
            pr_title_regex
        )
    };
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_set_default_qa_status_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_default_enable_qa(repo_owner, repo_name, status)
        .await?;

    let comment = if status {
        "QA disabled for this repository."
    } else {
        "QA enabled for this repository."
    };
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment.into()))
        .build())
}

pub async fn handle_set_default_checks_status_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_default_enable_checks(repo_owner, repo_name, status)
        .await?;

    let comment = if status {
        "Checks disabled for this repository."
    } else {
        "Checks enabled for this repository."
    };
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment.into()))
        .build())
}

/// Handle "Set needed reviewers" command.
pub async fn handle_set_needed_reviewers_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    count: u64,
) -> Result<CommandExecutionResult> {
    db_adapter
        .pull_requests()
        .set_needed_reviewers_count(repo_owner, repo_name, pr_number, count)
        .await?;

    let comment = format!("Needed reviewers count set to **{}** for this PR.", count);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_set_default_automerge_command(
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    value: bool,
) -> Result<CommandExecutionResult> {
    db_adapter
        .repositories()
        .set_default_automerge(repo_owner, repo_name, value)
        .await?;

    let comment = format!(
        "Default automerge status set to **{}** for this repository.",
        value
    );
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_admin_disable_command(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
) -> Result<CommandExecutionResult> {
    let repo_model = db_adapter
        .repositories()
        .get(repo_owner, repo_name)
        .await?
        .unwrap();
    if repo_model.manual_interaction() {
        StatusLogic::disable_validation_status(
            api_adapter,
            db_adapter,
            repo_owner,
            repo_name,
            pr_number,
        )
        .await?;
        db_adapter
            .pull_requests()
            .delete(repo_owner, repo_name, pr_number)
            .await?;

        let comment = "Bot disabled on this PR. Bye!";
        Ok(CommandExecutionResult::builder()
            .with_status_update(false)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment.into()))
            .build())
    } else {
        let comment = "You can not disable the bot on this PR, the repository is not in manual interaction mode.";
        Ok(CommandExecutionResult::builder()
            .denied()
            .with_status_update(false)
            .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
            .with_action(ResultAction::PostComment(comment.into()))
            .build())
    }
}

/// Handle `Help` command.
pub fn handle_help_command(
    config: &Config,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let comment = format!(
        "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
        You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
        \n\
        Supported commands:\n\
        - `noqa+`: _Skip QA validation_\n\
        - `noqa-`: _Enable QA validation_\n\
        - `qa+`: _Mark QA as passed_\n\
        - `qa-`: _Mark QA as failed_\n\
        - `qa?`: _Mark QA as waiting_\n\
        - `nochecks+`: _Skip checks validation_\n\
        - `nochecks-`: _Enable checks validation_\n\
        - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
        - `automerge-`: _Disable auto-merge for this PR_\n\
        - `lock+ <reason?>`: _Lock a pull-request (block merge)_\n\
        - `lock- <reason?>`: _Unlock a pull-request (unblock merge)_\n\
        - `req+ <reviewers>`: _Assign required reviewers (you can assign multiple reviewers)_\n\
        - `req- <reviewers>`: _Unassign required reviewers (you can unassign multiple reviewers)_\n\
        - `strategy+ <strategy>`: _Override merge strategy for this pull request_\n\
        - `strategy-`: _Remove the overriden merge strategy for this pull request_\n\
        - `merge <merge|squash|rebase?>`: _Try merging the pull request with optional strategy_\n\
        - `labels+ <label>`: _Set specific labels_\n\
        - `labels- <label>`: _Unset specific labels_\n\
        - `ping`: _Ping me_\n\
        - `gif <search>`: _Post a random GIF with a tag_\n\
        - `is-admin`: _Check if you are admin_\n\
        - `help`: _Show this comment_\n",
        comment_author, config.bot_username
    );

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `AdminHelp` command.
pub fn handle_admin_help_command(
    config: &Config,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let comment = format!(
        "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
        You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
        \n\
        Supported admin commands:\n\
        - `admin-help`: _Show this comment_\n\
        - `admin-enable`: _Enable me on a pull request with manual interaction_\n\
        - `admin-disable`: _Disable me on a pull request with manual interaction_\n\
        - `admin-set-default-needed-reviewers <count>`: _Set default needed reviewers count for this repository_\n\
        - `admin-set-default-merge-strategy <merge|squash|rebase>`: _Set default merge strategy for this repository_\n\
        - `admin-set-default-pr-title-regex <regex?>`: _Set default PR title validation regex for this repository_\n\
        - `admin-set-default-automerge+`: _Set automerge enabled for this repository_\n\
        - `admin-set-default-automerge-`: _Set automerge disabled for this repository_\n\
        - `admin-set-default-qa-status+`: _Enable QA validation by default for this repository_\n\
        - `admin-set-default-qa-status-`: _Disable QA validation by default for this repository_\n\
        - `admin-set-default-checks-status+`: _Enable checks validation by default for this repository_\n\
        - `admin-set-default-checks-status-`: _Disable checks validation by default for this repository_\n\
        - `admin-set-needed-reviewers <count>`: _Set needed reviewers count for this PR_\n\
        - `admin-reset-reviewers`: _Reset and update reviews on pull request (maintenance-type command)_\n\
        - `admin-reset-summary`: _Create a new summary message (maintenance-type command)_\n\
        - `admin-sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author, config.bot_username
    );

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database2::{
        Account, MockAccountDB, MockDbService, MockMergeRuleDB, MockPullRequestDB,
        MockRepositoryDB, MockRequiredReviewerDB, PullRequest, Repository,
    };
    use github_scbot_ghapi::{
        adapter::{GifFormat, GifObject, GifResponse, MediaObject, MockApiService},
        ApiError,
    };
    use github_scbot_types::common::GhUserPermission;
    use maplit::hashmap;
    use mockall::predicate;

    use super::*;
    use crate::commands::command::CommandHandlingStatus;

    #[actix_rt::test]
    async fn test_handle_auto_merge_command() -> Result<()> {
        let mut mock = Box::new(MockDbService::new());
        mock.expect_pull_requests().times(1).returning(move || {
            let mut pr = Box::new(MockPullRequestDB::new());
            pr.expect_set_automerge()
                .times(1)
                .with(
                    predicate::eq("me"),
                    predicate::eq("me"),
                    predicate::eq(1),
                    predicate::eq(true),
                )
                .returning(|_, _, _, _| async { Ok(PullRequest::default()) }.boxed());
            pr
        });

        // Automerge should be enabled
        let result = handle_auto_merge_command(mock.as_ref(), "me", "me", 1, "me", true).await?;
        assert!(result.should_update_status);
        assert_eq!(result.handling_status, CommandHandlingStatus::Handled);

        let mut mock = Box::new(MockDbService::new());
        mock.expect_pull_requests().times(1).returning(move || {
            let mut pr = Box::new(MockPullRequestDB::new());
            pr.expect_set_automerge()
                .times(1)
                .with(
                    predicate::eq("me"),
                    predicate::eq("me"),
                    predicate::eq(1),
                    predicate::eq(false),
                )
                .returning(|_, _, _, _| async { Ok(PullRequest::default()) }.boxed());
            pr
        });

        // Automerge should be disabled
        let result = handle_auto_merge_command(mock.as_ref(), "me", "me", 1, "me", false).await?;
        assert!(result.should_update_status);
        assert_eq!(result.handling_status, CommandHandlingStatus::Handled);

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_merge_command() -> Result<()> {
        let mut api_adapter = MockApiService::new();
        api_adapter
            .expect_pulls_merge()
            .times(0)
            .returning(|_, _, _, _, _, _| Ok(()));
        api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        api_adapter
            .expect_check_suites_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(3).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get().returning(|_, _, _| {
                async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });
        db_adapter.expect_repositories().times(3).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get().returning(|_, _| {
                async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });
        db_adapter
            .expect_required_reviewers()
            .times(3)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_list()
                    .returning(|_, _, _| async { Ok(vec![]) }.boxed());

                Box::new(mock)
            });
        db_adapter.expect_merge_rules().times(3).returning(|| {
            let mut mock = MockMergeRuleDB::new();
            mock.expect_get()
                .returning(|_, _, _, _| async { Ok(None) }.boxed());

            Box::new(mock)
        });

        // Merge should fail (wip)
        let upstream_pr = GhPullRequest {
            mergeable: Some(true),
            // Work in progress
            draft: true,
            ..Default::default()
        };
        let result = handle_merge_command(
            &api_adapter,
            &db_adapter,
            "me",
            "me",
            1,
            &upstream_pr,
            "me",
            None,
        )
        .await?;

        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::MinusOne),
                ResultAction::PostComment("Pull request is not ready to merge.".into())
            ]
        );

        // Merge should fail (GitHub error)
        let upstream_pr = GhPullRequest {
            mergeable: Some(true),
            ..Default::default()
        };

        let mut api_adapter = MockApiService::new();
        api_adapter
            .expect_pulls_merge()
            .times(1)
            .returning(|_, _, _, _, _, _| Err(ApiError::HTTPError("Nope.".into())));
        api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        api_adapter
            .expect_check_suites_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let result = handle_merge_command(
            &api_adapter,
            &db_adapter,
            "me",
            "me",
            1,
            &upstream_pr,
            "me",
            None,
        )
        .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::MinusOne),
                ResultAction::PostComment(
                    "Could not merge this pull request: _HTTP error: Nope._".into()
                )
            ]
        );

        // Merge should now work
        let mut api_adapter = MockApiService::new();
        api_adapter
            .expect_pulls_merge()
            .times(1)
            .returning(|_, _, _, _, _, _| Ok(()));
        api_adapter
            .expect_pull_reviews_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        api_adapter
            .expect_check_suites_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let result = handle_merge_command(
            &api_adapter,
            &db_adapter,
            "me",
            "me",
            1,
            &upstream_pr,
            "me",
            None,
        )
        .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::PlusOne),
                ResultAction::PostComment(
                    "Pull request successfully merged by me! (strategy: 'merge')".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_is_admin_command() -> Result<()> {
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().times(1).returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins()
                .returning(|| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        // Should not be admin
        let result = handle_is_admin_command(&db_adapter, "me").await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::MinusOne)]
        );

        let mut db_adapter = MockDbService::new();
        db_adapter.expect_accounts().times(1).returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins().returning(|| {
                async {
                    Ok(vec![Account::builder()
                        .username("me")
                        .is_admin(true)
                        .build()
                        .unwrap()])
                }
                .boxed()
            });

            Box::new(mock)
        });

        let result = handle_is_admin_command(&db_adapter, "me").await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::PlusOne)]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_admin_sync_command() -> Result<()> {
        let config = Config::from_env();

        let mut db_adapter = MockDbService::new();
        db_adapter.expect_repositories().times(2).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .returning(|_, _| async { Ok(None) }.boxed());
            mock.expect_create()
                .returning(|_| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });
        db_adapter.expect_pull_requests().times(2).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get()
                .returning(|_, _, _| async { Ok(None) }.boxed());
            mock.expect_create()
                .returning(|_| async { Ok(PullRequest::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = handle_admin_sync_command(&config, &db_adapter, "owner", "name", 1).await?;
        assert!(result.should_update_status);

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_skip_qa_command() -> Result<()> {
        // Skip.
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_qa_status()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(QaStatus::Skipped),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            Box::new(mock)
        });

        let result = handle_skip_qa_command(&db_adapter, "owner", "name", 1, "me", true).await?;
        assert!(result.should_update_status);

        // Reset.
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_qa_status()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(QaStatus::Waiting),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            Box::new(mock)
        });

        let result = handle_skip_qa_command(&db_adapter, "owner", "name", 1, "me", false).await?;
        assert!(result.should_update_status);

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_qa_command() -> Result<()> {
        // Approve.
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_qa_status()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(QaStatus::Pass),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            Box::new(mock)
        });

        let result = handle_qa_command(&db_adapter, "owner", "name", 1, "me", Some(true)).await?;
        assert!(result.should_update_status);

        // Unapprove.
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_qa_status()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(QaStatus::Fail),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            Box::new(mock)
        });

        let result = handle_qa_command(&db_adapter, "owner", "name", 1, "me", Some(false)).await?;
        assert!(result.should_update_status);

        // Reset.
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_set_qa_status()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                    predicate::eq(QaStatus::Waiting),
                )
                .returning(|_, _, _, _| {
                    async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                });
            Box::new(mock)
        });

        let result = handle_qa_command(&db_adapter, "owner", "name", 1, "me", None).await?;
        assert!(result.should_update_status);

        Ok(())
    }

    #[test]
    fn test_handle_ping_command() -> Result<()> {
        let result = handle_ping_command("me")?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("**me** pong!".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_gif_command() -> Result<()> {
        let config = Config::from_env();
        let mut api_adapter = MockApiService::new();
        api_adapter.expect_gif_search().times(1).returning(|_, _| {
            Ok(GifResponse {
                results: vec![GifObject {
                    media: vec![hashmap!(
                        GifFormat::Gif => MediaObject {
                            url: "http://url".into(),
                            size: Some(123)
                        }
                    )],
                }],
            })
        });

        // Valid GIF
        let result = handle_gif_command(&config, &api_adapter, "what").await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "![GIF](http://url)\n[_Via Tenor_](https://tenor.com/)".into()
                )
            ]
        );

        let mut api_adapter = MockApiService::new();
        api_adapter
            .expect_gif_search()
            .times(1)
            .returning(|_, _| Ok(GifResponse { results: vec![] }));

        // No GIFs
        let result = handle_gif_command(&config, &api_adapter, "what").await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("No compatible GIF found for query `what` :cry:".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_assign_required_reviewers_command() -> Result<()> {
        let mut api_adapter = MockApiService::new();
        let mut db_adapter = MockDbService::new();
        db_adapter.expect_pull_requests().times(1).returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get().times(1).returning(|_, _, _| {
                async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
            });

            Box::new(mock)
        });

        db_adapter
            .expect_required_reviewers()
            .times(4)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_get()
                    .returning(|_, _, _, _| async { Ok(None) }.boxed());
                mock.expect_create().returning(|_| {
                    async { Ok(RequiredReviewer::builder().build().unwrap()) }.boxed()
                });

                Box::new(mock)
            });

        api_adapter
            .expect_user_permissions_get()
            .times(2)
            .returning(|_, _, _| Ok(GhUserPermission::Write));
        api_adapter
            .expect_pull_reviewer_requests_add()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let reviewers = vec!["one".into(), "two".into()];
        let result = handle_assign_required_reviewers_command(
            &api_adapter,
            &db_adapter,
            "owner",
            "name",
            1,
            reviewers,
        )
        .await?;
        assert!(result.should_update_status);

        Ok(())
    }

    #[actix_rt::test]
    async fn test_handle_unassign_required_reviewers_command() -> Result<()> {
        let mut api_adapter = MockApiService::new();
        let mut db_adapter = MockDbService::new();
        db_adapter
            .expect_required_reviewers()
            .times(2)
            .returning(|| {
                let mut mock = MockRequiredReviewerDB::new();
                mock.expect_delete()
                    .times(1)
                    .returning(|_, _, _, _| async { Ok(true) }.boxed());

                Box::new(mock)
            });

        api_adapter
            .expect_pull_reviewer_requests_remove()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let reviewers = vec!["one".into(), "two".into()];
        let result = handle_unassign_required_reviewers_command(
            &api_adapter,
            &db_adapter,
            "owner",
            "name",
            1,
            reviewers,
        )
        .await?;
        assert!(result.should_update_status);

        Ok(())
    }

    #[test]
    fn test_handle_help_command() {
        let config = Config::from_env();
        let result = handle_help_command(&config, "me").unwrap();
        assert!(!result.should_update_status);
    }

    #[test]
    fn test_handle_admin_help_command() {
        let config = Config::from_env();
        let result = handle_admin_help_command(&config, "me").unwrap();
        assert!(!result.should_update_status);
    }
}
