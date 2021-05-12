//! Status module.

use github_scbot_api::{
    comments::{post_comment, update_comment},
    status::update_status_for_repository,
};
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn, DbPool,
};
use github_scbot_types::{
    labels::StepLabel,
    pulls::GhMergeStrategy,
    reviews::GhReviewState,
    status::{CheckStatus, QaStatus, StatusState},
};
use regex::Regex;
use tracing::debug;

use crate::{
    database::apply_pull_request_step,
    errors::Result,
    pulls::{
        determine_automatic_step, get_merge_strategy_for_branches, try_automerge_pull_request,
    },
};

/// Pull request status.
#[derive(Debug)]
pub struct PullRequestStatus {
    /// Approved reviewer usernames.
    pub approved_reviewers: Vec<String>,
    /// Automerge enabled?
    pub automerge: bool,
    /// Checks status.
    pub checks_status: CheckStatus,
    /// Checks URL.
    pub checks_url: String,
    /// Needed reviewers count.
    pub needed_reviewers_count: usize,
    /// QA status.
    pub qa_status: QaStatus,
    /// Missing required reviewers.
    pub missing_required_reviewers: Vec<String>,
    /// PR title is valid?
    pub valid_pr_title: bool,
    /// PR is locked?
    pub locked: bool,
    /// PR is in WIP?
    pub wip: bool,
}

impl PullRequestStatus {
    /// Create status from pull request.
    pub fn from_pull_request(
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        reviews: &[ReviewModel],
    ) -> Result<Self> {
        // Validate reviews
        let valid_reviews = Self::filter_valid_reviews(reviews);
        let needed_reviews = pr_model.needed_reviewers_count as usize;
        let mut approved_reviews = vec![];
        let mut required_reviews = vec![];

        for review in valid_reviews {
            let state = review.get_review_state();
            if review.required && state != GhReviewState::Approved {
                required_reviews.push(review.username.clone());
            } else if state == GhReviewState::Approved {
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
            wip: pr_model.wip,
        })
    }

    fn filter_valid_reviews(reviews: &[ReviewModel]) -> Vec<&ReviewModel> {
        reviews.iter().filter(|r| r.valid).collect()
    }

    /// Check if there are missing required reviews.
    pub fn missing_required_reviews(&self) -> bool {
        !self.missing_required_reviewers.is_empty()
    }

    /// Check if there are missing reviews.
    pub fn missing_reviews(&self) -> bool {
        self.missing_required_reviews()
            || self.needed_reviewers_count > self.approved_reviewers.len()
    }
}

/// Create or update status comment.
pub async fn create_or_update_status_comment(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<u64> {
    let conn = get_connection(&pool.clone())?;
    let reviews = pr_model.get_reviews(&conn)?;
    let strategy = get_merge_strategy_for_branches(
        &conn,
        repo_model,
        &pr_model.base_branch,
        &pr_model.head_branch,
    );
    let status_comment = generate_pr_status_comment(repo_model, pr_model, &reviews, strategy)?;

    // Try to lock the status comment ID
    if PullRequestModel::try_lock_status_comment_id(pr_model.id, pool.clone()).await? {
        post_new_status_comment(config, &conn, repo_model, pr_model, &status_comment).await
    } else {
        // Re-fetch comment ID
        let comment_id =
            PullRequestModel::fetch_status_comment_id(pr_model.id, pool.clone()).await? as u64;
        if comment_id > 0 {
            if let Ok(comment_id) = update_comment(
                config,
                &repo_model.owner,
                &repo_model.name,
                comment_id,
                &status_comment,
            )
            .await
            {
                Ok(comment_id)
            } else {
                // Comment ID is no more used on GitHub, recreate a new status
                post_new_status_comment(config, &conn, repo_model, pr_model, &status_comment).await
            }
        } else {
            // Too early, do not update the status comment
            Ok(0)
        }
    }
}

async fn post_new_status_comment(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment: &str,
) -> Result<u64> {
    let comment_id = post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        comment,
    )
    .await?;

    pr_model.set_status_comment_id(comment_id);
    pr_model.save(conn)?;
    Ok(comment_id)
}

/// Update pull request status.
pub async fn update_pull_request_status(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    commit_sha: &str,
) -> Result<()> {
    let conn = get_connection(&pool.clone())?;

    // Update step label.
    let reviews = pr_model.get_reviews(&conn)?;
    let step_label = determine_automatic_step(repo_model, pr_model, &reviews)?;
    pr_model.set_step_label(step_label);
    pr_model.save(&conn)?;

    apply_pull_request_step(config, repo_model, pr_model).await?;

    // Post status.
    create_or_update_status_comment(config, pool.clone(), repo_model, pr_model).await?;

    // Create or update status.
    let (status_state, status_title, status_message) =
        generate_pr_status_message(&repo_model, &pr_model, &reviews)?;
    update_status_for_repository(
        config,
        &repo_model.owner,
        &repo_model.name,
        commit_sha,
        status_state,
        status_title,
        &status_message,
    )
    .await?;

    // Merge if auto-merge is enabled
    if matches!(step_label, StepLabel::AwaitingMerge) && !pr_model.merged && pr_model.automerge {
        let result = try_automerge_pull_request(config, &conn, &repo_model, &pr_model).await?;
        if !result {
            pr_model.automerge = false;
            pr_model.save(&conn)?;

            // Update status
            create_or_update_status_comment(config, pool.clone(), repo_model, pr_model).await?;
        }
    }

    Ok(())
}

/// Generate status comment.
pub fn generate_pr_status_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
    strategy: GhMergeStrategy,
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
        rules_section = generate_status_comment_rule_section(repo_model, pr_model, strategy)?,
        checks_section = generate_status_comment_checks_section(&review_status, pr_model),
        config_section = generate_status_comment_config_section(pr_model),
        footer = generate_status_comment_footer(repo_model, pr_model)
    ))
}

/// Generate pull request status message.
pub fn generate_pr_status_message(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    reviews: &[ReviewModel],
) -> Result<(StatusState, &'static str, String)> {
    let status_title = "Validation";
    let mut status_state = StatusState::Success;
    let mut status_message = "All good.".to_string();
    let pr_status = PullRequestStatus::from_pull_request(repo_model, pr_model, reviews)?;

    debug!(
        pr_status = ?pr_status,
        message = "Generated pull request status"
    );

    if pr_status.wip {
        status_message = "PR is still in WIP".to_string();
        status_state = StatusState::Failure;
    } else if pr_status.valid_pr_title {
        // Check CI status
        match pr_status.checks_status {
            CheckStatus::Fail => {
                status_message = "Checks failed. Please fix.".to_string();
                status_state = StatusState::Failure;
            }
            CheckStatus::Waiting => {
                status_message = "Waiting for checks".to_string();
                status_state = StatusState::Pending;
            }
            CheckStatus::Pass | CheckStatus::Skipped => {
                // Check review status
                if !pr_status.missing_required_reviewers.is_empty() {
                    status_message = format!(
                        "Waiting on mandatory reviews ({})",
                        pr_status.missing_required_reviewers.join(", ")
                    );
                    status_state = StatusState::Pending;
                } else if pr_status.needed_reviewers_count > pr_status.approved_reviewers.len() {
                    status_message = "Waiting on reviews".to_string();
                    status_state = StatusState::Pending;
                } else {
                    // Check QA status
                    match pr_status.qa_status {
                        QaStatus::Fail => {
                            status_message = "QA failed. Please fix.".to_string();
                            status_state = StatusState::Failure;
                        }
                        QaStatus::Waiting => {
                            status_message = "Waiting for QA".to_string();
                            status_state = StatusState::Pending;
                        }
                        QaStatus::Pass | QaStatus::Skipped => {
                            if pr_status.locked {
                                status_message = "PR is locked".to_string();
                                status_state = StatusState::Failure;
                            }
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
    strategy: GhMergeStrategy,
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
        >   - _Rule:_ {rule}\n\
        > - :twisted_rightwards_arrows: **Merge strategy**: _{strategy}_\n",
        status = title_validation_status,
        rule = validation_regex,
        strategy = strategy.to_string()
    ))
}

fn generate_status_comment_checks_section(
    pull_request_status: &PullRequestStatus,
    pr_model: &PullRequestModel,
) -> String {
    let checks_message = match pr_model.get_checks_status() {
        CheckStatus::Pass => "_passed!_ :heavy_check_mark:",
        CheckStatus::Waiting => "_running..._ :clock2:",
        CheckStatus::Fail => "_failed._ :x:",
        CheckStatus::Skipped => "_skipped._ :heavy_check_mark:",
    };

    let qa_message = match pr_model.get_qa_status() {
        QaStatus::Pass => "_passed!_ :heavy_check_mark:",
        QaStatus::Waiting => "_waiting..._ :clock2:",
        QaStatus::Fail => "_failed._ :x:",
        QaStatus::Skipped => "_skipped._ :heavy_check_mark:",
    };

    let lock_message = if pr_model.locked {
        "Yes :x:"
    } else {
        "No :heavy_check_mark:"
    };

    let wip_message = if pr_model.wip {
        "Yes :x:"
    } else {
        "No :heavy_check_mark:"
    };

    let code_review_section = if pull_request_status.missing_required_reviews() {
        format!(
            "_waiting on mandatory reviews..._ ({}) :clock2:",
            pull_request_status.missing_required_reviewers.join(", ")
        )
    } else if pull_request_status.missing_reviews() {
        format!(
            "_waiting..._ ({} given / {} required) :clock2:",
            pull_request_status.approved_reviewers.len(),
            pull_request_status.needed_reviewers_count
        )
    } else {
        format!(
            "_passed! ({} given / {} required)_ :heavy_check_mark:",
            pull_request_status.approved_reviewers.len(),
            pull_request_status.needed_reviewers_count
        )
    };

    format!(
        ":speech_balloon: &mdash; **Status comment**\n\
        \n\
        > - :construction: **WIP?** {wip_message}\n\
        > - :checkered_flag: **Checks**: {checks_message}\n\
        > - :mag: **Code reviews**: {reviews_message}\n\
        > - :test_tube: **QA**: {qa_message}\n\
        > - :lock: **Locked?**: {lock_message}",
        wip_message = wip_message,
        checks_message = checks_message,
        reviews_message = code_review_section,
        qa_message = qa_message,
        lock_message = lock_message,
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
