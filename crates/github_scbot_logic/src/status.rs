//! Status module.

use github_scbot_api::{
    comments::{post_comment, update_comment},
    status::update_status_for_repository,
};
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn,
};
use github_scbot_types::{
    pull_requests::GHPullRequestReviewState,
    status::{CheckStatus, QAStatus, StatusState},
};
use regex::Regex;

use crate::{database::apply_pull_request_step, errors::Result};

/// Pull request status.
pub struct PullRequestStatus {
    /// Approved reviewer usernames.
    pub approved_reviewers: Vec<String>,
    /// Automerge enabled?
    pub automerge: bool,
    /// Checks status.
    pub checks_status: Option<CheckStatus>,
    /// Checks URL.
    pub checks_url: String,
    /// Needed reviewers count.
    pub needed_reviewers_count: usize,
    /// QA status.
    pub qa_status: Option<QAStatus>,
    /// Missing required reviewers.
    pub missing_required_reviewers: Vec<String>,
    /// PR title is valid?
    pub valid_pr_title: bool,
    /// PR is locked?
    pub locked: bool,
}

impl PullRequestStatus {
    /// Create status from pull request.
    ///
    /// # Arguments
    ///
    /// * `repo_model` - Repository model
    /// * `pr_model` - Pull request model
    /// * `reviews` - Review models
    pub fn from_pull_request(
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        reviews: &[ReviewModel],
    ) -> Result<Self> {
        // Validate reviews
        let needed_reviews = pr_model.needed_reviewers_count as usize;
        let mut approved_reviews = vec![];
        let mut required_reviews = vec![];

        for review in reviews {
            let state = review.get_review_state();
            if review.required && state != GHPullRequestReviewState::Approved {
                required_reviews.push(review.username.clone());
            } else if state == GHPullRequestReviewState::Approved {
                approved_reviews.push(review.username.clone());
            }
        }

        Ok(Self {
            approved_reviewers: approved_reviews,
            automerge: pr_model.automerge,
            checks_status: pr_model.get_checks_status(),
            checks_url: pr_model.get_checks_url(repo_model),
            needed_reviewers_count: needed_reviews,
            qa_status: pr_model.get_qa_status(),
            missing_required_reviewers: required_reviews,
            valid_pr_title: check_pr_title(repo_model, pr_model)?,
            locked: pr_model.locked,
        })
    }
}

/// Post status comment.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
pub async fn post_status_comment(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<u64> {
    let comment_id = pr_model.get_status_comment_id();
    let reviews = pr_model.get_reviews(conn)?;
    let status_comment = generate_pr_status_comment(repo_model, pr_model, &reviews)?;

    if comment_id > 0 {
        update_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id,
            &status_comment,
        )
        .await
        .map_err(Into::into)
    } else {
        let comment_id = post_comment(
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &status_comment,
        )
        .await?;

        pr_model.set_status_comment_id(comment_id);
        pr_model.save(conn)?;
        Ok(comment_id)
    }
}

/// Update pull request status.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `commit_sha` - Commit SHA.
pub async fn update_pull_request_status(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    commit_sha: &str,
) -> Result<()> {
    apply_pull_request_step(repo_model, pr_model).await?;
    post_status_comment(conn, repo_model, pr_model).await?;

    // Create or update status
    let reviews = pr_model.get_reviews(conn)?;
    let (status_state, status_title, status_message) =
        generate_pr_status_message(&repo_model, &pr_model, &reviews)?;
    update_status_for_repository(
        &repo_model.owner,
        &repo_model.name,
        commit_sha,
        status_state,
        status_title,
        &status_message,
    )
    .await?;

    Ok(())
}

/// Generate status comment.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviews` - Review models
pub fn generate_pr_status_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
) -> Result<String> {
    let review_status = PullRequestStatus::from_pull_request(repo_model, pr_model, reviews)?;

    Ok(format!(
        "_This is an auto-generated message summarizing this pull request._\n\
        \n\
        {rules_section}\n\
        \n\
        {checks_section}\n\
        \n\
        {config_section}\n\
        \n\
        {footer}",
        rules_section = generate_status_comment_rule_section(repo_model, pr_model)?,
        checks_section = generate_status_comment_checks_section(&review_status, pr_model),
        config_section = generate_status_comment_config_section(pr_model),
        footer = generate_status_comment_footer(repo_model, pr_model)
    ))
}

/// Generate pull request status message.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviews` - Review models
pub fn generate_pr_status_message(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
) -> Result<(StatusState, &'static str, String)> {
    let status_title = "Validation";
    let mut status_state = StatusState::Success;
    let mut status_message = "All good.".to_string();
    let pr_status = PullRequestStatus::from_pull_request(repo_model, pr_model, reviews)?;

    // Check PR title
    if pr_status.valid_pr_title {
        // Check CI status
        match pr_status.checks_status {
            Some(CheckStatus::Fail) => {
                status_message = "Checks failed. Please fix.".to_string();
                status_state = StatusState::Failure;
            }
            Some(CheckStatus::Waiting) | None => {
                status_message = "Waiting for checks".to_string();
                status_state = StatusState::Pending;
            }
            Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) => {
                // Check review status
                if !pr_status.missing_required_reviewers.is_empty() {
                    status_message = format!(
                        "Waiting on mandatory reviews ({:?})",
                        pr_status.missing_required_reviewers
                    );
                    status_state = StatusState::Pending;
                } else if pr_status.needed_reviewers_count > pr_status.approved_reviewers.len() {
                    status_message = "Waiting on reviews".to_string();
                    status_state = StatusState::Pending;
                } else {
                    // Check QA status
                    match pr_status.qa_status {
                        Some(QAStatus::Fail) => {
                            status_message = "QA failed. Please fix.".to_string();
                            status_state = StatusState::Failure;
                        }
                        Some(QAStatus::Waiting) | None => {
                            status_message = "Waiting for QA".to_string();
                            status_state = StatusState::Pending;
                        }
                        _ => {
                            // All good
                        }
                    }
                }
            }
        }
    } else {
        status_message = "PR title does not match regex.".to_string();
        status_state = StatusState::Failure;
    }

    Ok((status_state, status_title, status_message))
}

fn generate_status_comment_rule_section(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<String> {
    let validation_regex = if repo_model.pr_title_validation_regex.is_empty() {
        "None".to_owned()
    } else {
        format!("`{}`", repo_model.pr_title_validation_regex)
    };

    let title_validation_status = if check_pr_title(repo_model, pr_model)? {
        "_valid!_ :heavy_check_mark:"
    } else {
        "_invalid!_ :x:"
    };

    Ok(format!(
        ":pencil: &mdash; **Rules**\n\
        \n\
        > - :speech_balloon: **Title validation**: {status}\n\
        >   - _Rule:_ {rule}",
        status = title_validation_status,
        rule = validation_regex
    ))
}

fn generate_status_comment_checks_section(
    pull_request_status: &PullRequestStatus,
    pr_model: &PullRequestModel,
) -> String {
    let checks_message = match pr_model.get_checks_status() {
        Some(CheckStatus::Pass) => "_passed!_ :heavy_check_mark:",
        None | Some(CheckStatus::Waiting) => "_running..._ :clock2:",
        Some(CheckStatus::Fail) => "_failed._ :x:",
        Some(CheckStatus::Skipped) => "_skipped._ :heavy_check_mark:",
    };

    let qa_message = match pr_model.get_qa_status() {
        Some(QAStatus::Pass) => "_passed!_ :heavy_check_mark:",
        None | Some(QAStatus::Waiting) => "_waiting..._ :clock2:",
        Some(QAStatus::Fail) => "_failed._ :x:",
        Some(QAStatus::Skipped) => "_skipped._ :heavy_check_mark:",
    };

    let lock_message = if pr_model.locked {
        "Yes :x:"
    } else {
        "No :heavy_check_mark:"
    };

    let code_review_section = if pull_request_status.missing_required_reviewers.is_empty() {
        if pull_request_status.approved_reviewers.len()
            >= pull_request_status.needed_reviewers_count
        {
            format!(
                "_passed! ({} given / {} required)_ :heavy_check_mark:",
                pull_request_status.approved_reviewers.len(),
                pull_request_status.needed_reviewers_count
            )
        } else {
            format!(
                "_waiting..._ ({} given / {} required) :clock2:",
                pull_request_status.approved_reviewers.len(),
                pull_request_status.needed_reviewers_count
            )
        }
    } else {
        format!(
            "_waiting on mandatory reviews..._ ({:?}) :clock2:",
            pull_request_status.missing_required_reviewers
        )
    };

    format!(
        ":speech_balloon: &mdash; **Status comment**\n\
        \n\
        > - :checkered_flag: **Checks**: {checks_message}\n\
        > - :mag: **Code reviews**: {reviews_message}\n\
        > - :test_tube: **QA**: {qa_message}\n\
        > - :lock: **Locked?**: {lock_message}",
        checks_message = checks_message,
        reviews_message = code_review_section,
        qa_message = qa_message,
        lock_message = lock_message
    )
}

fn generate_status_comment_config_section(pr_model: &PullRequestModel) -> String {
    let automerge_message = if pr_model.automerge {
        ":heavy_check_mark:"
    } else {
        ":x:"
    };

    format!(
        ":gear: &mdash; **Configuration**\n\
        \n\
        > - :twisted_rightwards_arrows: **Automerge**: {automerge}",
        automerge = automerge_message
    )
}

fn generate_status_comment_footer(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> String {
    format!(
        "[_See checks output by clicking this link :triangular_flag_on_post:_]({checks_url})",
        checks_url = pr_model.get_checks_url(repo_model)
    )
}

fn check_pr_title(repo_model: &RepositoryModel, pr_model: &PullRequestModel) -> Result<bool> {
    if repo_model.pr_title_validation_regex.is_empty() {
        Ok(true)
    } else {
        Regex::new(&repo_model.pr_title_validation_regex)
            .map(|rgx| rgx.is_match(&pr_model.name))
            .map_err(Into::into)
    }
}
