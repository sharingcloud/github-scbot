//! Status

use regex::Regex;

use crate::{
    api::comments::{post_comment_for_repo, update_comment},
    database::models::QAStatus,
};
use crate::{
    api::status::StatusState,
    database::models::{CheckStatus, DbConn, PullRequestModel, RepositoryModel},
};
use crate::{types::PullRequestReviewState, webhook::errors::Result};

struct ReviewStatus {
    needed_count: usize,
    approved_reviewers: Vec<String>,
    still_required_reviewers: Vec<String>,
}

impl ReviewStatus {
    pub fn get_review_message(&self) -> String {
        if !self.still_required_reviewers.is_empty() {
            format!(
                "_waiting on mandatory reviews_ ({:?}) :clock2:",
                self.still_required_reviewers
            )
        } else if self.needed_count > self.approved_reviewers.len() {
            "_waiting..._ :clock2:".to_string()
        } else {
            "_todo_".to_string()
        }
    }
}

#[allow(clippy::cast_sign_loss)]
pub async fn post_status_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<u64> {
    let review_status = get_review_status(conn, pr_model)?;
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
        review_status.get_review_message(),
        qa_message,
        automerge_message,
        pr_model.get_checks_url(repo_model)
    );

    if comment_id > 0 {
        update_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id as u64,
            &status_comment,
        )
        .await
        .map_err(Into::into)
    } else {
        let comment_id =
            post_comment_for_repo(repo_model, pr_model.number as u64, &status_comment).await?;
        pr_model.update_status_comment(conn, comment_id)?;
        Ok(comment_id)
    }
}

#[allow(clippy::cast_sign_loss)]
fn get_review_status(conn: &DbConn, pr_model: &PullRequestModel) -> Result<ReviewStatus> {
    // Validate reviews
    let needed_reviews = pr_model.needed_reviewers_count as usize;
    let mut approved_reviews = vec![];
    let mut required_reviews = vec![];

    let reviews = pr_model.get_reviews(conn)?;
    for review in &reviews {
        let state = review.state_enum();
        if review.required && state != PullRequestReviewState::Approved {
            required_reviews.push(review.username.clone());
        } else if state == PullRequestReviewState::Approved {
            approved_reviews.push(review.username.clone());
        }
    }

    Ok(ReviewStatus {
        needed_count: needed_reviews,
        approved_reviewers: approved_reviews,
        still_required_reviewers: required_reviews,
    })
}

pub fn generate_pr_status(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<(StatusState, &'static str, String)> {
    let status_title = "Validation";
    let mut status_state = StatusState::Success;
    let mut status_message = "All good.".to_string();

    // Try to validate PR title
    if !repo_model.pr_title_validation_regex.is_empty() {
        let rgx = Regex::new(&repo_model.pr_title_validation_regex)?;
        if !rgx.is_match(&pr_model.name) {
            status_message = "PR title does not match regex.".to_string();
            status_state = StatusState::Failure;
        }
    }

    if status_state == StatusState::Success {
        // Validate checks
        match pr_model.check_status_enum() {
            Some(CheckStatus::Fail) => {
                status_message = "Checks failed. Please fix.".to_string();
                status_state = StatusState::Failure;
            }
            Some(CheckStatus::Waiting) | None => {
                status_message = "Waiting for checks".to_string();
                status_state = StatusState::Pending;
            }
            Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) => (),
        }
    }

    if status_state == StatusState::Success {
        // Validate reviews
        let review_status = get_review_status(conn, pr_model)?;

        if !review_status.still_required_reviewers.is_empty() {
            status_message = format!(
                "Waiting on mandatory reviews ({:?})",
                review_status.still_required_reviewers
            );
            status_state = StatusState::Pending;
        } else if review_status.needed_count > review_status.approved_reviewers.len() {
            status_message = "Waiting on reviews".to_string();
            status_state = StatusState::Pending;
        }
    }

    if status_state == StatusState::Success {
        // Validate QA
        match pr_model.qa_status_enum() {
            Some(QAStatus::Fail) => {
                status_message = "QA failed. Please fix.".to_string();
                status_state = StatusState::Failure;
            }
            Some(QAStatus::Waiting) | None => {
                status_message = "Waiting for QA".to_string();
                status_state = StatusState::Pending;
            }
            _ => (),
        }
    }

    Ok((status_state, status_title, status_message))
}
