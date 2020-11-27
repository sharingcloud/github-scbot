//! Webhook logic

use std::convert::TryInto;

use eyre::Result;

use super::constants::{ENV_BOT_USERNAME, ENV_DISABLE_WELCOME_COMMENTS};
use super::types::{PullRequest, Repository};
use crate::api::labels::set_step_label;
use crate::database::models::{
    CheckStatus, DbConn, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
};
use crate::{
    api::comments::{post_comment, update_comment},
    database::models::QAStatus,
};

pub fn process_repository(conn: &DbConn, repo: &Repository) -> Result<RepositoryModel> {
    RepositoryModel::get_or_create(
        conn,
        &RepositoryCreation {
            name: &repo.name,
            owner: &repo.owner.login,
        },
    )
}

pub fn process_pull_request(
    conn: &DbConn,
    repo: &Repository,
    pull: &PullRequest,
) -> Result<(RepositoryModel, PullRequestModel)> {
    let repo = process_repository(conn, repo)?;
    let pr = PullRequestModel::get_or_create(
        conn,
        &PullRequestCreation {
            repository_id: repo.id,
            name: &pull.title,
            number: pull.number.try_into()?,
            automerge: false,
            check_status: CheckStatus::Pass.as_str(),
            step: "none",
        },
    )?;

    Ok((repo, pr))
}

pub async fn post_welcome_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    if std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        post_comment(
            &repo_model.owner,
            &repo_model.name,
            pr_model.number.try_into()?,
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                pr_author
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn post_status_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<u64> {
    let comment_id = pr_model.status_comment_id;
    let check_status = pr_model.check_status_enum();
    let (checks_passed, checks_icon, checks_message) = match check_status {
        Some(CheckStatus::Pass) => (true, ":heavy_check_mark:", "_passed!_ :tada:"),
        Some(CheckStatus::Waiting) => (false, ":clock2:", "_running..._ :gear:"),
        Some(CheckStatus::Fail) => (false, ":x:", "_failed._ :boom:"),
        _ => (true, ":heavy_check_mark:", "_skipped._"),
    };

    let mut status_comment = format!(
        "**Status comment**\n\n\
        {} &mdash; :checkered_flag: **Checks**: {}\n\
        {} &mdash; :mag: **Code reviews**: _waiting_\n\
        {} &mdash; :test_tube: **QA**: _waiting_\n",
        checks_icon, checks_message, ":clock2:", ":clock2:",
    );

    if !checks_passed {
        status_comment = format!(
            "{}\n\n\
            [_See checks output by clicking this link :triangular_flag_on_post:_]({})",
            status_comment,
            pr_model.get_checks_url(repo_model)
        );
    }

    if comment_id > 0 {
        update_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id.try_into()?,
            &status_comment,
        )
        .await
    } else {
        post_comment(
            &repo_model.owner,
            &repo_model.name,
            pr_model.number.try_into()?,
            &status_comment,
        )
        .await
    }
}

pub async fn apply_pull_request_step(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    set_step_label(
        &repo_model.owner,
        &repo_model.name,
        pr_model.number.try_into()?,
        pr_model.step_enum(),
    )
    .await
}

#[derive(Debug, PartialEq)]
pub enum CommentAction {
    SkipQAStatus(bool),
    QAStatus(bool),
    AutoMergeStatus(bool),
}

impl CommentAction {
    pub fn from_comment(comment: &str) -> Option<Self> {
        Some(match comment {
            "noqa+" => Self::SkipQAStatus(true),
            "noqa-" => Self::SkipQAStatus(false),
            "qa+" => Self::QAStatus(true),
            "qa-" => Self::QAStatus(false),
            "automerge+" => Self::AutoMergeStatus(true),
            "automerge-" => Self::AutoMergeStatus(false),
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

pub async fn parse_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    line: &str,
) -> Result<()> {
    if let Some(command_line) = parse_command_string_from_comment_line(line) {
        let action = CommentAction::from_comment(command_line);
        let mut status_updated = false;

        match action {
            Some(CommentAction::AutoMergeStatus(s)) => {
                pr_model.update_automerge(conn, s)?;
                status_updated = true;
                let status_text = if s { "enabled" } else { "disabled" };
                let comment = format!("Automerge {} by @{}", status_text, comment_author);
                post_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.number.try_into()?,
                    &comment,
                )
                .await?;
            }
            Some(CommentAction::QAStatus(s)) => {
                let (status, status_text) = if s {
                    (QAStatus::Pass, "marked as pass")
                } else {
                    (QAStatus::Fail, "marked as fail")
                };

                pr_model.update_qa_status(conn, Some(status))?;
                status_updated = true;
                let comment = format!("QA is {} by @{}", status_text, comment_author);
                post_comment(
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.number.try_into()?,
                    &comment,
                )
                .await?;
            }
            _ => (),
        }

        if status_updated {
            post_status_comment(repo_model, pr_model).await?;
        }
    }

    Ok(())
}

pub fn parse_command_string_from_comment_line(comment: &str) -> Option<&str> {
    let bot_username = std::env::var(ENV_BOT_USERNAME).unwrap_or_else(|_| "SCBot".to_string());

    if comment.starts_with(&bot_username) {
        let (_, command) = comment.split_at(bot_username.len());
        return Some(command.trim());
    }

    None
}
