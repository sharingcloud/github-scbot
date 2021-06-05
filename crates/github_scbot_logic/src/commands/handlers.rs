use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::{
    IDatabaseAdapter, PullRequestModel, RepositoryModel, ReviewModel,
};
use github_scbot_types::{
    issues::GhReactionType, labels::StepLabel, pulls::GhMergeStrategy, status::QaStatus,
};
use tracing::info;

use super::command::CommandExecutionResult;
use crate::{
    auth::{is_admin, list_known_admin_usernames},
    commands::command::ResultAction,
    errors::Result,
    gif::GifPoster,
    pulls::synchronize_pull_request,
    status::{determine_automatic_step, disable_validation_status, PullRequestStatus},
};

/// Handle `Automerge` command.
pub async fn handle_auto_merge_command(
    db_adapter: &dyn IDatabaseAdapter,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    pr_model.automerge = status;
    db_adapter.pull_request().save(pr_model).await?;

    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by **{}**", status_text, comment_author);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::PostComment(comment))
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `Merge` command.
pub async fn handle_merge_command(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    // Use step to determine merge possibility
    let pr_status = PullRequestStatus::from_database(db_adapter, repo_model, pr_model).await?;
    let step = determine_automatic_step(&pr_status)?;
    let commit_title = pr_model.get_merge_commit_title();

    let mut actions = vec![];

    if matches!(step, StepLabel::AwaitingMerge) {
        if let Err(e) = api_adapter
            .pulls_merge(
                &repo_model.owner,
                &repo_model.name,
                pr_model.get_number(),
                &commit_title,
                "",
                pr_status.merge_strategy,
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
                comment_author,
                pr_status.merge_strategy.to_string()
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
    db_adapter: &dyn IDatabaseAdapter,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let known_admins = list_known_admin_usernames(db_adapter).await?;
    let status = is_admin(comment_author, &known_admins);
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
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<CommandExecutionResult> {
    let (pr, _sha) = synchronize_pull_request(
        config,
        api_adapter,
        db_adapter,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
    )
    .await?;
    *pr_model = pr;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `SkipQA` command.
pub async fn handle_skip_qa_command(
    db_adapter: &dyn IDatabaseAdapter,
    pr_model: &mut PullRequestModel,
    status: bool,
) -> Result<CommandExecutionResult> {
    if status {
        pr_model.set_qa_status(QaStatus::Skipped);
    } else {
        pr_model.set_qa_status(QaStatus::Waiting);
    }

    db_adapter.pull_request().save(pr_model).await?;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `QaStatus` command.
pub async fn handle_qa_command(
    db_adapter: &dyn IDatabaseAdapter,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: Option<bool>,
) -> Result<CommandExecutionResult> {
    let (status, status_text) = match status {
        Some(true) => (QaStatus::Pass, "marked as pass"),
        Some(false) => (QaStatus::Fail, "marked as fail"),
        None => (QaStatus::Waiting, "marked as waiting"),
    };

    pr_model.set_qa_status(status);
    db_adapter.pull_request().save(pr_model).await?;

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
    api_adapter: &impl IAPIAdapter,
    search_terms: &str,
) -> Result<CommandExecutionResult> {
    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(
            GifPoster::generate_random_gif_comment(config, api_adapter, search_terms).await?,
        ))
        .build())
}

/// Handle `AssignRequiredReviewers` command.
pub async fn handle_assign_required_reviewers_command(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    info!(
        pull_request_number = pr_model.get_number(),
        reviewers = ?reviewers,
        message = "Request required reviewers",
    );

    // Communicate to GitHub
    api_adapter
        .pull_reviewer_requests_add(
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &reviewers,
        )
        .await?;

    for reviewer in &reviewers {
        ReviewModel::builder(repo_model, pr_model, reviewer)
            .required(true)
            .create_or_update(db_adapter.review())
            .await?;
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `UnassignRequiredReviewers` command.
pub async fn handle_unassign_required_reviewers_command(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    info!(
        pull_request_number = pr_model.get_number(),
        reviewers = ?reviewers,
        message = "Remove required reviewers",
    );

    api_adapter
        .pull_reviewer_requests_remove(
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &reviewers,
        )
        .await?;

    for reviewer in &reviewers {
        ReviewModel::builder(repo_model, pr_model, reviewer)
            .required(false)
            .create_or_update(db_adapter.review())
            .await?;
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `Lock` command.
pub async fn handle_lock_command(
    db_adapter: &dyn IDatabaseAdapter,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
    reason: Option<String>,
) -> Result<CommandExecutionResult> {
    let status_text = if status { "locked" } else { "unlocked" };

    pr_model.locked = status;
    db_adapter.pull_request().save(pr_model).await?;

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
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &mut RepositoryModel,
    count: u32,
) -> Result<CommandExecutionResult> {
    repo_model.default_needed_reviewers_count = count as i32;
    db_adapter.repository().save(repo_model).await?;

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
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &mut RepositoryModel,
    strategy: GhMergeStrategy,
) -> Result<CommandExecutionResult> {
    repo_model.set_default_merge_strategy(strategy);
    db_adapter.repository().save(repo_model).await?;

    let comment = format!(
        "Merge strategy set to **{}** for this repository.",
        strategy.to_string()
    );
    Ok(CommandExecutionResult::builder()
        .with_status_update(false)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle "Set default PR title regex" command.
pub async fn handle_set_default_pr_title_regex_command(
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &mut RepositoryModel,
    pr_title_regex: String,
) -> Result<CommandExecutionResult> {
    repo_model.pr_title_validation_regex = pr_title_regex.clone();
    db_adapter.repository().save(repo_model).await?;

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

/// Handle "Set needed reviewers" command.
pub async fn handle_set_needed_reviewers_command(
    db_adapter: &dyn IDatabaseAdapter,
    pr_model: &mut PullRequestModel,
    count: u32,
) -> Result<CommandExecutionResult> {
    pr_model.needed_reviewers_count = count as i32;
    db_adapter.pull_request().save(pr_model).await?;

    let comment = format!("Needed reviewers count set to **{}** for this PR.", count);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

pub async fn handle_admin_disable_command(
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    repo_model: &mut RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<CommandExecutionResult> {
    if repo_model.manual_interaction {
        db_adapter.pull_request().remove(pr_model).await?;
        disable_validation_status(api_adapter, db_adapter, repo_model, pr_model).await?;

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
            .with_status_update(true)
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
        - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
        - `automerge-`: _Disable auto-merge for this PR_\n\
        - `lock+ <reason?>`: _Lock a pull-request (block merge)_\n\
        - `lock- <reason?>`: _Unlock a pull-request (unblock merge)_\n\
        - `req+ <reviewers>`: _Assign required reviewers (you can assign multiple reviewers)_\n\
        - `req- <reviewers>`: _Unassign required reviewers (you can unassign multiple reviewers)_\n\
        - `merge`: _Try merging the pull request_\n\
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
        - `admin-set-needed-reviewers <count>`: _Set needed reviewers count for this PR_\n\
        - `admin-sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author, config.bot_username
    );

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}
