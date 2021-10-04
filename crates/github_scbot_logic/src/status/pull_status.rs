use github_scbot_database::models::{
    IDatabaseAdapter, MergeRuleModel, PullRequestModel, RepositoryModel, ReviewModel,
};
use github_scbot_types::{
    pulls::GhMergeStrategy,
    reviews::GhReviewState,
    status::{CheckStatus, QaStatus},
};
use regex::Regex;

use crate::errors::Result;

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
    /// Pull request title regex.
    pub pull_request_title_regex: String,
    /// PR title is valid?
    pub valid_pr_title: bool,
    /// PR is locked?
    pub locked: bool,
    /// PR is in WIP?
    pub wip: bool,
    /// Merge strategy
    pub merge_strategy: GhMergeStrategy,
}

impl PullRequestStatus {
    /// Create status from pull request and database.
    pub async fn from_database(
        db_adapter: &dyn IDatabaseAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
    ) -> Result<Self> {
        let reviews = pr_model.reviews(db_adapter.review()).await?;
        let strategy = if let Some(s) = pr_model.strategy_override() {
            s
        } else {
            MergeRuleModel::get_strategy_from_branches(
                db_adapter.merge_rule(),
                repo_model,
                pr_model.base_branch(),
                pr_model.head_branch(),
            )
            .await
        };

        Self::from_pull_request(repo_model, pr_model, &reviews, strategy)
    }

    /// Create status from pull request.
    pub fn from_pull_request(
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        reviews: &[ReviewModel],
        strategy: GhMergeStrategy,
    ) -> Result<Self> {
        // Validate reviews
        let valid_reviews = Self::filter_valid_reviews(reviews);
        let needed_reviews = pr_model.needed_reviewers_count() as usize;
        let mut approved_reviews = vec![];
        let mut required_reviews = vec![];

        for review in valid_reviews {
            let state = review.state();
            if review.required() && state != GhReviewState::Approved {
                required_reviews.push(review.username().into());
            } else if state == GhReviewState::Approved {
                approved_reviews.push(review.username().into());
            }
        }

        Ok(Self {
            approved_reviewers: approved_reviews,
            automerge: pr_model.automerge(),
            checks_status: pr_model.check_status(),
            checks_url: pr_model.checks_url(repo_model),
            pull_request_title_regex: repo_model.pr_title_validation_regex().into(),
            needed_reviewers_count: needed_reviews,
            qa_status: pr_model.qa_status(),
            missing_required_reviewers: required_reviews,
            valid_pr_title: Self::check_pr_title(
                pr_model.name(),
                repo_model.pr_title_validation_regex(),
            )?,
            locked: pr_model.locked(),
            wip: pr_model.wip(),
            merge_strategy: strategy,
        })
    }

    fn filter_valid_reviews(reviews: &[ReviewModel]) -> Vec<&ReviewModel> {
        reviews.iter().filter(|r| r.valid()).collect()
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
