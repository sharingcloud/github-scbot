//! Comments API module

use std::convert::TryInto;

use eyre::Result;

use super::constants::ENV_DISABLE_WELCOME_COMMENTS;
use super::get_client;
use crate::database::models::{CheckStatus, PullRequestModel, RepositoryModel};

async fn post_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    body: &str,
) -> Result<u64> {
    if cfg!(test) {
        // No comment
        Ok(0)
    } else {
        let client = get_client().await?;

        let final_body = format!("{}\n\n_Beep boop, i'm a bot!_ :robot:", body);

        let comment = client
            .issues(repo_owner, repo_name)
            .create_comment(pr_number, final_body)
            .await?;

        Ok(comment.id)
    }
}

async fn update_comment(
    repo_owner: &str,
    repo_name: &str,
    comment_id: u64,
    body: &str,
) -> Result<u64> {
    if !cfg!(test) {
        let client = get_client().await?;

        client
            .issues(repo_owner, repo_name)
            .update_comment(comment_id, body)
            .await?;
    }

    Ok(comment_id)
}

pub async fn post_welcome_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    pr_author: &str,
) -> Result<()> {
    if std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        post_comment(
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
    }

    Ok(())
}

pub async fn create_or_update_status_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<u64> {
    let comment_id = pr_model.status_comment_id;
    let check_status = pr_model.check_status_enum()?;
    let (checks_passed, checks_message) = match check_status {
        CheckStatus::Pass => (true, "_passed!_ :tada:"),
        CheckStatus::Waiting => (false, "_running..._ :clock2:"),
        CheckStatus::Fail => (false, "_failed._ :boom:"),
    };

    let mut status_comment = format!(
        "**Status comment**\n\
        - [{}] :checkered_flag: **Checks**: {}\n\
        - [{}] :mag: **Code reviews**: _waiting_\n\
        - [{}] :test_tube: **QA**: _waiting_\n",
        if checks_passed { "x" } else { " " },
        checks_message,
        " ",
        " "
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
