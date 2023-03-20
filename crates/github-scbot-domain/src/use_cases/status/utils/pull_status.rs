use std::collections::HashSet;

use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{
    ChecksStatus, MergeStrategy, PullRequest, QaStatus, Repository, RequiredReviewer,
};
use github_scbot_ghapi_interface::{
    reviews::ReviewApi,
    types::{GhPullRequest, GhReview, GhReviewState},
    ApiService,
};
use regex::Regex;

use crate::{
    errors::Result,
    use_cases::{
        checks::DetermineChecksStatusUseCase, pulls::DeterminePullRequestMergeStrategyUseCase,
    },
};

/// Pull request status.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PullRequestStatus {
    /// Reviewers waiting for changes
    pub changes_required_reviewers: Vec<String>,
    /// Approved reviewer usernames.
    pub approved_reviewers: Vec<String>,
    /// Automerge enabled?
    pub automerge: bool,
    /// Checks status.
    pub checks_status: ChecksStatus,
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
    pub merge_strategy: MergeStrategy,
}

impl PullRequestStatus {
    /// Create status from pull request and database.
    #[tracing::instrument(
        skip_all,
        fields(
            repo_owner = %repo_owner,
            repo_name = %repo_name,
            pr_number = pr_number
        )
    )]
    pub async fn from_database(
        api_service: &dyn ApiService,
        db_service: &dyn DbService,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u64,
        upstream_pr: &GhPullRequest,
    ) -> Result<Self> {
        let repo_model = db_service
            .repositories_get(repo_owner, repo_name)
            .await?
            .unwrap();
        let pr_model = db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
            .unwrap();

        let upstream_reviews =
            ReviewApi::list_reviews_for_pull_request(api_service, repo_owner, repo_name, pr_number)
                .await?;
        let required_reviewers = db_service
            .required_reviewers_list(repo_owner, repo_name, pr_number)
            .await?;
        let checks_status = if pr_model.checks_enabled {
            DetermineChecksStatusUseCase {
                api_service,
                repo_owner,
                repo_name,
                commit_sha: &upstream_pr.head.sha,
                wait_for_initial_checks: pr_model.checks_enabled,
            }
            .run()
            .await?
        } else {
            ChecksStatus::Skipped
        };

        let strategy = if let Some(s) = pr_model.strategy_override {
            s
        } else {
            let base_branch = &upstream_pr.base.reference;
            let head_branch = &upstream_pr.head.reference;
            DeterminePullRequestMergeStrategyUseCase {
                db_service,
                repo_owner,
                repo_name,
                head_branch,
                base_branch,
                default_strategy: repo_model.default_strategy,
            }
            .run()
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
        strategy: MergeStrategy,
        required_reviewers: Vec<RequiredReviewer>,
        checks_status: ChecksStatus,
        upstream_reviews: Vec<GhReview>,
        upstream_pr: &GhPullRequest,
    ) -> Result<Self> {
        // Validate reviews
        let needed_reviews = pr_model.needed_reviewers_count as usize;
        let mut approved_reviews = vec![];
        let mut required_reviews = vec![];
        let mut changes_required_reviews = vec![];

        // Required reviewers may not be in upstream reviews,
        // we need to make sure they are parsed as well.
        let mut seen_reviewers = HashSet::new();
        for review in upstream_reviews {
            let username = review.user.login;
            let required = Self::is_required_reviewer(&required_reviewers, &username);
            let state = review.state;
            let approved = state == GhReviewState::Approved;
            seen_reviewers.insert(username.clone());

            if state == GhReviewState::ChangesRequested {
                changes_required_reviews.push(username);
            } else if required && !approved {
                required_reviews.push(username);
            } else if approved {
                approved_reviews.push(username);
            }
        }

        for required_reviewer in required_reviewers {
            if !seen_reviewers.contains(&required_reviewer.username) {
                required_reviews.push(required_reviewer.username.to_string());
            }
        }

        Ok(Self {
            changes_required_reviewers: changes_required_reviews,
            approved_reviewers: approved_reviews,
            automerge: pr_model.automerge,
            checks_status,
            checks_url: Self::get_checks_url(&repo_model.owner, &repo_model.name, pr_model.number),
            pull_request_title_regex: repo_model.pr_title_validation_regex.clone(),
            needed_reviewers_count: needed_reviews,
            qa_status: pr_model.qa_status,
            missing_required_reviewers: required_reviews,
            valid_pr_title: Self::check_pr_title(
                &upstream_pr.title,
                &repo_model.pr_title_validation_regex,
            )?,
            locked: pr_model.locked,
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
        required_reviewers.iter().any(|r| r.username == username)
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

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_ghapi_interface::{
        review::{GhReviewApi, GhReviewStateApi},
        types::{GhBranch, GhUser},
        MockApiService,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn blank_no_checks_no_qa_no_reviewers() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                default_enable_checks: false,
                default_enable_qa: false,
                default_needed_reviewers_count: 0,
                ..Default::default()
            })
            .await
            .unwrap();

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            ..Default::default()
        };

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        let status = PullRequestStatus::from_database(
            &api_service,
            &db_service,
            "me",
            "test",
            1,
            &upstream_pr,
        )
        .await
        .unwrap();

        assert_eq!(
            status,
            PullRequestStatus {
                checks_url: "https://github.com/me/test/pull/1/checks".into(),
                needed_reviewers_count: 0,
                checks_status: ChecksStatus::Skipped,
                qa_status: QaStatus::Skipped,
                valid_pr_title: true,
                mergeable: true,
                changes_required_reviewers: vec![],
                approved_reviewers: vec![],
                missing_required_reviewers: vec![],
                automerge: false,
                locked: false,
                merged: false,
                wip: false,
                merge_strategy: MergeStrategy::Merge,
                pull_request_title_regex: String::new()
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_no_qa_no_reviewers() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                default_enable_checks: true,
                default_enable_qa: false,
                default_needed_reviewers_count: 0,
                ..Default::default()
            })
            .await
            .unwrap();

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_check_runs_list()
            .once()
            .withf(|owner, name, sha| owner == "me" && name == "test" && sha == "abcdef")
            .return_once(|_, _, _| Ok(vec![]));

        let status = PullRequestStatus::from_database(
            &api_service,
            &db_service,
            "me",
            "test",
            1,
            &upstream_pr,
        )
        .await
        .unwrap();

        assert_eq!(
            status,
            PullRequestStatus {
                checks_url: "https://github.com/me/test/pull/1/checks".into(),
                needed_reviewers_count: 0,
                checks_status: ChecksStatus::Waiting,
                qa_status: QaStatus::Skipped,
                valid_pr_title: true,
                mergeable: true,
                changes_required_reviewers: vec![],
                approved_reviewers: vec![],
                missing_required_reviewers: vec![],
                automerge: false,
                locked: false,
                merged: false,
                wip: false,
                merge_strategy: MergeStrategy::Merge,
                pull_request_title_regex: String::new()
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_qa_no_reviewers() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                default_enable_checks: true,
                default_enable_qa: true,
                default_needed_reviewers_count: 0,
                ..Default::default()
            })
            .await
            .unwrap();

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_check_runs_list()
            .once()
            .withf(|owner, name, sha| owner == "me" && name == "test" && sha == "abcdef")
            .return_once(|_, _, _| Ok(vec![]));

        let status = PullRequestStatus::from_database(
            &api_service,
            &db_service,
            "me",
            "test",
            1,
            &upstream_pr,
        )
        .await
        .unwrap();

        assert_eq!(
            status,
            PullRequestStatus {
                checks_url: "https://github.com/me/test/pull/1/checks".into(),
                needed_reviewers_count: 0,
                checks_status: ChecksStatus::Waiting,
                qa_status: QaStatus::Waiting,
                valid_pr_title: true,
                mergeable: true,
                changes_required_reviewers: vec![],
                approved_reviewers: vec![],
                missing_required_reviewers: vec![],
                automerge: false,
                locked: false,
                merged: false,
                wip: false,
                merge_strategy: MergeStrategy::Merge,
                pull_request_title_regex: String::new()
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_qa_reviewers() {
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();

        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                default_enable_checks: true,
                default_enable_qa: true,
                default_needed_reviewers_count: 2,
                ..Default::default()
            })
            .await
            .unwrap();

        let _ = db_service
            .pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            head: GhBranch {
                sha: "abcdef".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
            .return_once(|_, _, _| {
                Ok(vec![GhReviewApi {
                    state: GhReviewStateApi::Approved,
                    user: GhUser {
                        login: "dummy".into(),
                    },
                    ..Default::default()
                }])
            });

        api_service
            .expect_check_runs_list()
            .once()
            .withf(|owner, name, sha| owner == "me" && name == "test" && sha == "abcdef")
            .return_once(|_, _, _| Ok(vec![]));

        let status = PullRequestStatus::from_database(
            &api_service,
            &db_service,
            "me",
            "test",
            1,
            &upstream_pr,
        )
        .await
        .unwrap();

        assert_eq!(
            status,
            PullRequestStatus {
                checks_url: "https://github.com/me/test/pull/1/checks".into(),
                needed_reviewers_count: 2,
                checks_status: ChecksStatus::Waiting,
                qa_status: QaStatus::Waiting,
                valid_pr_title: true,
                mergeable: true,
                changes_required_reviewers: vec![],
                approved_reviewers: vec!["dummy".into()],
                missing_required_reviewers: vec![],
                automerge: false,
                locked: false,
                merged: false,
                wip: false,
                merge_strategy: MergeStrategy::Merge,
                pull_request_title_regex: String::new()
            }
        )
    }
}
