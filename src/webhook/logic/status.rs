//! Status

use std::convert::TryInto;

use regex::Regex;
use tracing::info;

use crate::errors::Result;
use crate::{
    api::comments::{post_comment_for_repo, update_comment},
    database::models::QAStatus,
};
use crate::{
    api::status::StatusState,
    database::models::{CheckStatus, DbConn, PullRequestModel, RepositoryModel},
};

pub async fn post_status_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<u64> {
    let comment_id = pr_model.status_comment_id;
    let checks_message = match pr_model.check_status_enum() {
        Some(CheckStatus::Pass) => "_passed!_ :heavy_check_mark:",
        None | Some(CheckStatus::Waiting) => "_running..._ :clock2:",
        Some(CheckStatus::Fail) => "_failed._ :x:",
        Some(CheckStatus::Skipped) => "_skipped._ :heavy_check_mark:",
    };

    let qa_message = match pr_model.qa_status_enum() {
        Some(QAStatus::Pass) => "_passed!_ :heavy_check_mark:",
        None | Some(QAStatus::Waiting) => "_waiting..._ :clock2:",
        Some(QAStatus::Fail) => "_failed._ :x:",
        Some(QAStatus::Skipped) => "_skipped._ :heavy_check_mark:",
    };

    let automerge_message = if pr_model.automerge {
        ":heavy_check_mark:"
    } else {
        ":x:"
    };

    let validation_regex = if repo_model.pr_title_validation_regex.is_empty() {
        "None".to_owned()
    } else {
        format!("`{}`", repo_model.pr_title_validation_regex)
    };

    let status_comment = format!(
        "_This is an auto-generated message summarizing this pull request._\n\
        \n\
        :pencil: &mdash; **Rules**\n\
        \n\
        > - :speech_balloon: **Title validation**: ???\n\
        >   - _Rule:_ {}\n\
        \n\
        :speech_balloon: &mdash; **Status comment**\n\
        \n\
        > - :checkered_flag: **Checks**: {}\n\
        > - :mag: **Code reviews**: {}\n\
        > - :test_tube: **QA**: {}\n\
        \n\
        :gear: &mdash; **Configuration**\n\
        \n\
        > - :twisted_rightwards_arrows: **Automerge**: {}\n\
        \n\
        [_See checks output by clicking this link :triangular_flag_on_post:_]({})",
        validation_regex,
        checks_message,
        "_waiting..._ :clock2:",
        qa_message,
        automerge_message,
        pr_model.get_checks_url(repo_model)
    );

    if comment_id > 0 {
        update_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id.try_into()?,
            &status_comment,
        )
        .await
    } else {
        let comment_id =
            post_comment_for_repo(repo_model, pr_model.number.try_into()?, &status_comment).await?;
        pr_model.update_status_comment(conn, comment_id)?;
        Ok(comment_id)
    }
}

pub fn generate_pr_status(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<(StatusState, &'static str, &'static str)> {
    let status_title = "Validation";
    let mut status_state = StatusState::Success;
    let mut status_message = "All good.";

    info!("Generating PR status...");

    // Try to validate PR title
    if !repo_model.pr_title_validation_regex.is_empty() {
        let rgx = Regex::new(&repo_model.pr_title_validation_regex)?;
        if !rgx.is_match(&pr_model.name) {
            status_message = "PR title does not match regex.";
            status_state = StatusState::Failure;

            info!("> {:?} {}", status_state, status_message);
        }
    }

    if status_state == StatusState::Success {
        // Validate checks
        match pr_model.check_status_enum() {
            Some(CheckStatus::Fail) => {
                status_message = "Checks failed. Please fix.";
                status_state = StatusState::Failure;
                info!("> {:?} {}", status_state, status_message);
            }
            Some(CheckStatus::Waiting) | None => {
                status_message = "Waiting for checks";
                status_state = StatusState::Pending;
                info!("> {:?} {}", status_state, status_message);
            }
            Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) => (),
        }
    }

    if status_state == StatusState::Success {
        // Validate QA
        match pr_model.qa_status_enum() {
            Some(QAStatus::Fail) => {
                status_message = "QA failed. Please fix.";
                status_state = StatusState::Failure;
                info!("> {:?} {}", status_state, status_message);
            }
            Some(QAStatus::Waiting) | None => {
                status_message = "Waiting for QA";
                status_state = StatusState::Pending;
                info!("> {:?} {}", status_state, status_message);
            }
            _ => (),
        }
    }

    // TODO: Validate reviews

    Ok((status_state, status_title, status_message))
}
