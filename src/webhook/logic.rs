//! Webhook logic

use std::convert::TryInto;

use eyre::Result;

use super::constants::ENV_DISABLE_WELCOME_COMMENTS;
use super::types::{PullRequest, Repository};
use crate::api::comments::{post_comment, update_comment};
use crate::api::labels::set_step_label;
use crate::database::models::{
    CheckStatus, DbConn, PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel,
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
