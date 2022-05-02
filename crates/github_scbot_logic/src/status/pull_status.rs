use github_scbot_database2::{DbService, PullRequest, Repository, RequiredReviewer};
use github_scbot_ghapi::{adapter::ApiService, reviews::ReviewApi};
use github_scbot_types::{
    pulls::{GhMergeStrategy, GhPullRequest},
    reviews::{GhReview, GhReviewState},
    status::{CheckStatus, QaStatus},
};
use regex::Regex;

use crate::{errors::Result, pulls::PullRequestLogic};

/// Pull request status.
#[derive(Debug)]
pub struct PullRequestStatus {
    /// Reviewers waiting for changes
    pub changes_required_reviewers: Vec<String>,
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
    /// Pull request title regex.
    pub pull_request_title_regex: String,
    /// PR title is valid?
    pub valid_pr_title: bool,
    /// PR is locked?
    pub locked: bool,
    /// PR is in WIP?
    pub wip: bool,
    /// PR is mergeable?
    pub mergeable: bool,
    /// PR is merged?,
    pub merged: bool,
    /// Merge strategy
    pub merge_strategy: GhMergeStrategy,
}

impl PullRequestStatus {
    /// Create status from pull request and database.
    #[tracing::instrument(skip(api_adapter, db_adapter), ret)]
    pub async fn from_database(
        api_adapter: &dyn ApiService,
        db_adapter: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<Self> {
        let repo_model = db_adapter
            .repositories()
            .get(repo_owner, repo_name)
            .await?
            .unwrap();
        let pr_model = db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
            .await?
            .unwrap();

        let upstream_reviews =
            ReviewApi::list_reviews_for_pull_request(api_adapter, repo_owner, repo_name, pr_number)
                .await?;
        let required_reviewers = db_adapter
            .required_reviewers()
            .list(repo_owner, repo_name, pr_number)
            .await?;
        let checks_status = PullRequestLogic::get_checks_status_from_github(
            api_adapter,
            repo_owner,
            repo_name,
            &upstream_pr.head.sha,
            pr_model.checks_enabled(),
            &[],
        )
        .await?;

        let strategy = if let Some(s) = pr_model.strategy_override() {
            *s
        } else {
            let base_branch = &upstream_pr.base.reference;
            let head_branch = &upstream_pr.head.reference;
            Self::get_strategy_from_branches(
                db_adapter,
                repo_owner,
                repo_name,
                base_branch,
                head_branch,
                repo_model.default_strategy(),
            )
            .await?
        };

        Self::from_pull_request(
            &repo_model,
            &pr_model,
            strategy,
            required_reviewers,
            checks_status,
            upstream_reviews,
            upstream_pr,
        )
    }

    fn from_pull_request(
        repo_model: &Repository,
        pr_model: &PullRequest,
        strategy: GhMergeStrategy,
        required_reviewers: Vec<RequiredReviewer>,
        checks_status: CheckStatus,
        upstream_reviews: Vec<GhReview>,
        upstream_pr: &GhPullRequest,
    ) -> Result<Self> {
        // Validate reviews
        let needed_reviews = pr_model.needed_reviewers_count() as usize;
        let mut approved_reviews = vec![];
        let mut required_reviews = vec![];
        let mut changes_required_reviews = vec![];

        for review in upstream_reviews {
            let username = review.user.login;
            let required = Self::is_required_reviewer(&required_reviewers, &username);
            let state = review.state;
            let approved = state == GhReviewState::Approved;

            if state == GhReviewState::ChangesRequested {
                changes_required_reviews.push(username);
            } else if required && !approved {
                required_reviews.push(username);
            } else if approved {
                approved_reviews.push(username);
            }
        }

        Ok(Self {
            changes_required_reviewers: changes_required_reviews,
            approved_reviewers: approved_reviews,
            automerge: pr_model.automerge(),
            checks_status,
            checks_url: Self::get_checks_url(
                repo_model.owner(),
                repo_model.name(),
                pr_model.number(),
            ),
            pull_request_title_regex: repo_model.pr_title_validation_regex().into(),
            needed_reviewers_count: needed_reviews,
            qa_status: *pr_model.qa_status(),
            missing_required_reviewers: required_reviews,
            valid_pr_title: Self::check_pr_title(
                &upstream_pr.title,
                repo_model.pr_title_validation_regex(),
            )?,
            locked: pr_model.locked(),
            wip: upstream_pr.draft,
            mergeable: upstream_pr.mergeable.unwrap_or(true),
            merged: upstream_pr.merged.unwrap_or(false),
            merge_strategy: strategy,
        })
    }

    /// Get checks url.
    pub fn get_checks_url(owner: &str, name: &str, number: u64) -> String {
        format!("https://github.com/{owner}/{name}/pull/{number}/checks")
    }

    /// Check if a reviewer is required.
    pub fn is_required_reviewer(required_reviewers: &[RequiredReviewer], username: &str) -> bool {
        required_reviewers.iter().any(|r| r.username() == username)
    }

    /// Get merge strategy for base and head branches.
    pub async fn get_strategy_from_branches(
        db_adapter: &dyn DbService,
        owner: &str,
        name: &str,
        base_branch: &str,
        head_branch: &str,
        default_strategy: GhMergeStrategy,
    ) -> Result<GhMergeStrategy> {
        match db_adapter
            .merge_rules()
            .get(owner, name, base_branch.into(), head_branch.into())
            .await?
        {
            Some(r) => Ok(r.strategy()),
            None => Ok(default_strategy),
        }
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

    /// Check if changes are required.
    pub fn changes_required(&self) -> bool {
        !self.changes_required_reviewers.is_empty()
    }

    /// Check PR title
    fn check_pr_title(name: &str, pattern: &str) -> Result<bool> {
        if pattern.is_empty() {
            Ok(true)
        } else {
            Regex::new(pattern)
                .map(|rgx| rgx.is_match(name))
                .map_err(Into::into)
        }
    }
}
