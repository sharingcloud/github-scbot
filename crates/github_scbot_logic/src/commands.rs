//! Commands module.

use github_scbot_api::{
    comments::{add_reaction_to_comment, post_comment},
    pulls::{get_pull_request_sha, merge_pull_request},
    reviews::{remove_reviewers_for_pull_request, request_reviewers_for_pull_request},
};
use github_scbot_core::constants::ENV_BOT_USERNAME;
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel, ReviewCreation, ReviewModel},
    DbConn,
};
use github_scbot_types::{
    issues::GHReactionType,
    labels::StepLabel,
    pull_requests::GHMergeStrategy,
    status::{CheckStatus, QAStatus},
};
use tracing::info;

use super::{errors::Result, status::update_pull_request_status};
use crate::pull_requests::determine_automatic_step;

/// Command handling status.
#[derive(Debug, Clone, Copy)]
pub enum CommandHandlingStatus {
    /// Command handled.
    Handled,
    /// Command ignored.
    Ignored,
}

/// Command.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Skip QA status.
    SkipQAStatus(bool),
    /// Enable/Disable QA status.
    QAStatus(Option<bool>),
    /// Enable/Disable checks status.
    ChecksStatus(bool),
    /// Enable/Disable automerge.
    Automerge(bool),
    /// Assign required reviewers.
    AssignRequiredReviewers(Vec<String>),
    /// Unassign required reviewers.
    UnassignRequiredReviewers(Vec<String>),
    /// Add/Remove lock with optional reason.
    Lock(bool, Option<String>),
    /// Merge pull request.
    Merge,
    /// Ping the bot.
    Ping,
    /// Show help message.
    Help,
    /// Synchronize status.
    Synchronize,
}

impl Command {
    /// Create a command from a comment and arguments.
    ///
    /// # Arguments
    ///
    /// * `comment` - Comment
    /// * `args` - Arguments
    pub fn from_comment(comment: &str, args: &[&str]) -> Option<Self> {
        Some(match comment {
            "noqa+" => Self::SkipQAStatus(true),
            "noqa-" => Self::SkipQAStatus(false),
            "qa+" => Self::QAStatus(Some(true)),
            "qa-" => Self::QAStatus(Some(false)),
            "qa?" => Self::QAStatus(None),
            "checks+" => Self::ChecksStatus(true),
            "checks-" => Self::ChecksStatus(false),
            "automerge+" => Self::Automerge(true),
            "automerge-" => Self::Automerge(false),
            "merge" => Self::Merge,
            "req+" => Self::AssignRequiredReviewers(Self::parse_reviewers(args)),
            "req-" => Self::UnassignRequiredReviewers(Self::parse_reviewers(args)),
            "lock+" => Self::Lock(true, Self::parse_message(args)),
            "lock-" => Self::Lock(false, Self::parse_message(args)),
            "help" => Self::Help,
            "ping" => Self::Ping,
            "sync" => Self::Synchronize,
            _ => return None,
        })
    }

    fn parse_message(args: &[&str]) -> Option<String> {
        if args.is_empty() {
            None
        } else {
            Some(args.join(" "))
        }
    }

    fn parse_reviewers(reviewers: &[&str]) -> Vec<String> {
        reviewers
            .iter()
            .filter_map(|x| x.strip_prefix('@').map(str::to_string))
            .collect()
    }
}

/// Parse commands from comment body.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
/// * `comment_body` - Comment body
pub async fn parse_commands(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
    comment_body: &str,
) -> Result<CommandHandlingStatus> {
    let mut command_handling = CommandHandlingStatus::Ignored;

    for line in comment_body.lines() {
        let line_handling =
            parse_single_command(conn, repo_model, pr_model, comment_id, comment_author, line)
                .await?;
        if matches!(line_handling, CommandHandlingStatus::Handled) {
            command_handling = line_handling;
        }
    }

    Ok(command_handling)
}

/// Parse command from a single comment line.
///
/// # Arguments
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
/// * `line` - Comment line
pub async fn parse_single_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
    line: &str,
) -> Result<CommandHandlingStatus> {
    let mut command_handled = CommandHandlingStatus::Ignored;

    if let Some((command_line, args)) = parse_command_string_from_comment_line(line) {
        let action = Command::from_comment(command_line, &args);
        info!(
            "Interpreting action {:?} from author {} on repository {}, PR #{}",
            action,
            comment_author,
            repo_model.get_path(),
            pr_model.get_number()
        );

        command_handled = CommandHandlingStatus::Handled;
        let status_updated = match action {
            Some(Command::Automerge(s)) => {
                handle_auto_merge_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(Command::SkipQAStatus(s)) => handle_skip_qa_command(conn, pr_model, s)?,
            Some(Command::QAStatus(s)) => {
                handle_qa_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(Command::ChecksStatus(s)) => {
                handle_checks_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(Command::Lock(s, reason)) => {
                handle_lock_command(conn, repo_model, pr_model, comment_author, s, reason).await?
            }
            Some(Command::Ping) => {
                handle_ping_command(repo_model, pr_model, comment_author).await?
            }
            Some(Command::Merge) => {
                handle_merge_command(conn, repo_model, pr_model, comment_id, comment_author).await?
            }
            Some(Command::Synchronize) => true,
            Some(Command::AssignRequiredReviewers(reviewers)) => {
                handle_assign_required_reviewers_command(conn, repo_model, pr_model, reviewers)
                    .await?
            }
            Some(Command::UnassignRequiredReviewers(reviewers)) => {
                handle_unassign_required_reviewers_command(conn, repo_model, pr_model, reviewers)
                    .await?
            }
            Some(Command::Help) => {
                handle_help_command(repo_model, pr_model, comment_author).await?
            }
            _ => {
                command_handled = CommandHandlingStatus::Ignored;
                false
            }
        };

        if status_updated {
            let sha =
                get_pull_request_sha(&repo_model.owner, &repo_model.name, pr_model.get_number())
                    .await?;
            update_pull_request_status(conn, repo_model, pr_model, &sha).await?;
        }
    }

    Ok(command_handled)
}

/// Parse command string from comment line.
///
/// # Arguments
///
/// * `comment` - Comment
pub fn parse_command_string_from_comment_line(comment: &str) -> Option<(&str, Vec<&str>)> {
    if let Ok(bot_username) = std::env::var(ENV_BOT_USERNAME) {
        if comment.starts_with(&bot_username) {
            // Plus one for the '@' symbol
            let (_, command) = comment.split_at(bot_username.len());
            let mut split = command.trim().split_whitespace();

            if let Some(command) = split.next() {
                // Take command and remaining args
                return Some((command, split.collect()));
            }
        }
    }

    None
}

/// Handle `Automerge` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - Automerge status
pub async fn handle_auto_merge_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<bool> {
    pr_model.automerge = status;
    pr_model.save(conn)?;

    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by @{}", status_text, comment_author);
    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Merge` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
pub async fn handle_merge_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
) -> Result<bool> {
    // Use step to determine merge possibility
    let reviews = pr_model.get_reviews(conn)?;
    let step = determine_automatic_step(repo_model, pr_model, &reviews)?;
    let commit_title = format!("{} (#{})", pr_model.name, pr_model.get_number());

    if matches!(step, StepLabel::AwaitingMerge) {
        match merge_pull_request(
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &commit_title,
            "",
            GHMergeStrategy::Squash,
        )
        .await
        {
            Err(e) => {
                add_reaction_to_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GHReactionType::MinusOne,
                )
                .await?;
                post_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.get_number(),
                    &format!("Could not merge this pull request: {}", e),
                )
                .await?;
            }
            _ => {
                add_reaction_to_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GHReactionType::PlusOne,
                )
                .await?;
                post_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.get_number(),
                    &format!("Pull request successfully merged by @{}!", comment_author),
                )
                .await?;
            }
        }
    } else {
        add_reaction_to_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id,
            GHReactionType::MinusOne,
        )
        .await?;
        post_comment(
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            "Pull request is not ready to merge.",
        )
        .await?;
    }

    Ok(true)
}

/// Handle `SkipQA` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pr_model` - Pull request model
/// * `status` - Skip QA status
pub fn handle_skip_qa_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    status: bool,
) -> Result<bool> {
    if status {
        pr_model.set_qa_status(QAStatus::Skipped);
    } else {
        pr_model.set_qa_status(QAStatus::Waiting);
    }

    pr_model.save(conn)?;

    Ok(true)
}

/// Handle `QAStatus` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - QA status
pub async fn handle_qa_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: Option<bool>,
) -> Result<bool> {
    let (status, status_text) = match status {
        Some(true) => (QAStatus::Pass, "marked as pass"),
        Some(false) => (QAStatus::Fail, "marked as fail"),
        None => (QAStatus::Waiting, "marked as waiting"),
    };

    pr_model.set_qa_status(status);
    pr_model.save(conn)?;

    let comment = format!("QA is {} by @{}", status_text, comment_author);
    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `ChecksStatus` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - QA status
pub async fn handle_checks_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<bool> {
    let (status, status_text) = if status {
        (CheckStatus::Pass, "marked as pass")
    } else {
        (CheckStatus::Fail, "marked as fail")
    };

    pr_model.set_checks_status(status);
    pr_model.save(conn)?;

    let comment = format!("Checks are {} by @{}", status_text, comment_author);
    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Ping` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
pub async fn handle_ping_command(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    comment_author: &str,
) -> Result<bool> {
    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &format!("@{} pong!", comment_author),
    )
    .await?;

    Ok(true)
}

/// Handle `AssignRequiredReviewers` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviewers` - Reviewers
pub async fn handle_assign_required_reviewers_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    info!(
        "Request required reviewers for PR #{}: {:#?}",
        pr_model.get_number(),
        reviewers
    );

    // Communicate to GitHub
    request_reviewers_for_pull_request(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &reviewers,
    )
    .await?;

    for reviewer in &reviewers {
        let mut entry = ReviewModel::get_or_create(
            conn,
            ReviewCreation {
                pull_request_id: pr_model.id,
                username: reviewer,
                ..Default::default()
            },
        )?;

        entry.required = true;
        entry.save(conn)?;
    }

    Ok(true)
}

/// Handle `UnassignRequiredReviewers` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviewers` - Reviewers
pub async fn handle_unassign_required_reviewers_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    info!(
        "Remove required reviewers for PR #{}: {:#?}",
        pr_model.get_number(),
        reviewers
    );

    remove_reviewers_for_pull_request(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &reviewers,
    )
    .await?;

    for reviewer in &reviewers {
        let mut entry = ReviewModel::get_or_create(
            conn,
            ReviewCreation {
                pull_request_id: pr_model.id,
                username: reviewer,
                ..Default::default()
            },
        )?;

        entry.required = false;
        entry.save(conn)?;
    }

    Ok(true)
}

/// Handle `Lock` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - Lock status
/// * `reason` - Optional lock motivation
pub async fn handle_lock_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
    reason: Option<String>,
) -> Result<bool> {
    let status_text = if status { "locked" } else { "unlocked" };

    pr_model.locked = status;
    pr_model.save(conn)?;

    let mut comment = format!("Pull request {} by @{}", status_text, comment_author);
    if let Some(reason) = reason {
        comment = format!("{}\n**Reason**: {}", comment, reason);
    }

    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Help` command.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
pub async fn handle_help_command(
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
) -> Result<bool> {
    let comment = format!(
        "Hello @{} ! I am a GitHub helper bot ! :robot:\n\
        You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
        \n\
        Supported commands:\n\
        - `noqa+`: _Skip QA validation_\n\
        - `noqa-`: _Enable QA validation_\n\
        - `qa+`: _Mark QA as passed_\n\
        - `qa-`: _Mark QA as failed_\n\
        - `qa?`: _Mark QA as waiting_\n\
        - `checks+`: _Mark checks as passed_\n\
        - `checks-`: _Mark checks as failed_\n\
        - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
        - `automerge-`: _Disable auto-merge for this PR_\n\
        - `lock+ <reason?>`: _Lock a pull-request (block merge)_\n\
        - `lock- <reason?>`: _Unlock a pull-request (unblock merge)_\n\
        - `req+ <reviewers>`: _Assign required reviewers (you can assign multiple reviewers)_\n\
        - `req- <reviewers>`: _Unassign required reviewers (you can unassign multiple reviewers)_\n\
        - `help`: _Show this comment_\n\
        - `ping`: _Ping me._\n\
        - `sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author,
        std::env::var(ENV_BOT_USERNAME).unwrap()
    );

    post_comment(
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(false)
}
