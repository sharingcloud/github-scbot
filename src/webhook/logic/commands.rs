//! Commands

use std::convert::TryInto;

use tracing::info;

use crate::database::models::{CheckStatus, DbConn, PullRequestModel, RepositoryModel};
use crate::errors::Result;
use crate::webhook::constants::ENV_BOT_USERNAME;
use crate::webhook::logic::status::{generate_pr_status, post_status_comment};
use crate::{
    api::{
        comments::post_comment_for_repo, pulls::get_pull_request_sha,
        status::update_status_for_repo,
    },
    database::models::QAStatus,
};

#[derive(Debug, PartialEq)]
pub enum CommentAction {
    SkipQAStatus(bool),
    QAStatus(bool),
    ChecksStatus(bool),
    AutoMergeStatus(bool),
    AddReviewers(Vec<String>),
    RemoveReviewers(Vec<String>),
    Ping,
    Help,
    Synchronize,
}

impl CommentAction {
    fn parse_reviewers(reviewers: &[&str]) -> Vec<String> {
        reviewers
            .iter()
            .filter_map(|x| x.strip_prefix('@').map(str::to_string))
            .collect()
    }

    pub fn from_comment(comment: &str, args: &[&str]) -> Option<Self> {
        Some(match comment {
            "noqa+" => Self::SkipQAStatus(true),
            "noqa-" => Self::SkipQAStatus(false),
            "qa+" => Self::QAStatus(true),
            "qa-" => Self::QAStatus(false),
            "checks+" => Self::ChecksStatus(true),
            "checks-" => Self::ChecksStatus(false),
            "automerge+" => Self::AutoMergeStatus(true),
            "automerge-" => Self::AutoMergeStatus(false),
            "req+" => Self::AddReviewers(Self::parse_reviewers(args)),
            "req-" => Self::RemoveReviewers(Self::parse_reviewers(args)),
            "help" => Self::Help,
            "ping" => Self::Ping,
            "sync" => Self::Synchronize,
            _ => return None,
        })
    }
}

pub async fn parse_issue_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    comment: &str,
) -> Result<()> {
    for line in comment.lines() {
        parse_comment(conn, repo_model, pr_model, comment_author, line).await?;
    }

    Ok(())
}

pub async fn handle_auto_merge_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<bool> {
    pr_model.update_automerge(conn, status)?;
    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by @{}", status_text, comment_author);
    post_comment_for_repo(repo_model, pr_model.number.try_into()?, &comment).await?;

    Ok(true)
}

pub fn handle_skip_qa_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    status: bool,
) -> Result<bool> {
    if status {
        pr_model.update_qa_status(conn, Some(QAStatus::Skipped))?;
    } else {
        pr_model.update_qa_status(conn, Some(QAStatus::Waiting))?;
    }

    pr_model.update_step_auto(conn)?;

    Ok(true)
}

pub async fn handle_qa_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<bool> {
    let (status, status_text) = if status {
        (QAStatus::Pass, "marked as pass")
    } else {
        (QAStatus::Fail, "marked as fail")
    };

    pr_model.update_qa_status(conn, Some(status))?;
    pr_model.update_step_auto(conn)?;

    let comment = format!("QA is {} by @{}", status_text, comment_author);
    post_comment_for_repo(repo_model, pr_model.number.try_into()?, &comment).await?;

    Ok(true)
}

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

    pr_model.update_check_status(conn, Some(status))?;
    pr_model.update_step_auto(conn)?;
    let comment = format!("Checks are {} by @{}", status_text, comment_author);
    post_comment_for_repo(repo_model, pr_model.number.try_into()?, &comment).await?;

    Ok(true)
}

pub async fn handle_ping_command(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    comment_author: &str,
) -> Result<bool> {
    post_comment_for_repo(
        repo_model,
        pr_model.number.try_into()?,
        &format!("@{} pong!", comment_author),
    )
    .await?;

    Ok(false)
}

pub async fn handle_assign_reviewers_command(
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    use crate::api::reviews::request_reviewers_for_pr;

    info!(
        "Request reviewers for PR #{}: {:#?}",
        pr_model.number, reviewers
    );
    request_reviewers_for_pr(repo_model, pr_model, &reviewers).await?;

    Ok(false)
}

pub async fn handle_unassign_reviewers_command(
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    use crate::api::reviews::remove_reviewers_for_pr;

    info!(
        "Remove reviewers for PR #{}: {:#?}",
        pr_model.number, reviewers
    );
    remove_reviewers_for_pr(repo_model, pr_model, &reviewers).await?;

    Ok(false)
}

pub async fn handle_sync_command(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<bool> {
    post_status_comment(conn, repo_model, pr_model).await?;

    Ok(true)
}

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
        - `checks+`: _Mark checks as passed_\n\
        - `checks-`: _Mark checks as failed_\n\
        - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
        - `automerge-`: _Disable auto-merge for this PR_\n\
        - `req+`: _Assign reviewers (you can assign multiple reviewers)_\n\
        - `req-`: _Unassign reviewers (you can unassign multiple reviewers)_\n\
        - `help`: _Show this comment_\n\
        - `ping`: _Ping me._\n\
        - `sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author,
        std::env::var(ENV_BOT_USERNAME).unwrap()
    );

    post_comment_for_repo(repo_model, pr_model.number.try_into()?, &comment).await?;

    Ok(false)
}

pub async fn parse_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    line: &str,
) -> Result<()> {
    if let Some((command_line, args)) = parse_command_string_from_comment_line(line) {
        let action = CommentAction::from_comment(command_line, &args);
        let status_updated = match action {
            Some(CommentAction::AutoMergeStatus(s)) => {
                handle_auto_merge_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(CommentAction::SkipQAStatus(s)) => handle_skip_qa_command(conn, pr_model, s)?,
            Some(CommentAction::QAStatus(s)) => {
                handle_qa_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(CommentAction::ChecksStatus(s)) => {
                handle_checks_command(conn, repo_model, pr_model, comment_author, s).await?
            }
            Some(CommentAction::Ping) => {
                handle_ping_command(repo_model, pr_model, comment_author).await?
            }
            Some(CommentAction::Synchronize) => {
                handle_sync_command(conn, repo_model, pr_model).await?
            }
            Some(CommentAction::AddReviewers(reviewers)) => {
                handle_assign_reviewers_command(repo_model, pr_model, reviewers).await?
            }
            Some(CommentAction::RemoveReviewers(reviewers)) => {
                handle_unassign_reviewers_command(repo_model, pr_model, reviewers).await?
            }
            Some(CommentAction::Help) => {
                handle_help_command(repo_model, pr_model, comment_author).await?
            }
            _ => false,
        };

        if status_updated {
            post_status_comment(conn, repo_model, pr_model).await?;

            // Update status checks
            let sha = get_pull_request_sha(
                &repo_model.owner,
                &repo_model.name,
                pr_model.number.try_into()?,
            )
            .await?;

            // Create or update status
            let (status_state, status_title, status_message) =
                generate_pr_status(&repo_model, &pr_model)?;
            update_status_for_repo(
                &repo_model,
                &sha,
                status_state,
                status_title,
                status_message,
            )
            .await?;
        }
    }

    Ok(())
}

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
