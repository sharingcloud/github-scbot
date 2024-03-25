use std::collections::HashSet;

use prbot_ghapi_interface::{
    reviews::ReviewApi,
    types::{GhPullRequest, GhReviewState},
};
use prbot_models::{
    ChecksStatus, MergeStrategy, PullRequestHandle, PullRequestRule, QaStatus, RequiredReviewer,
};
use regex::Regex;
use shaku::HasComponent;

use crate::{
    errors::Result,
    use_cases::{
        checks::DetermineChecksStatusInterface,
        pulls::{DeterminePullRequestMergeStrategyInterface, ResolvePullRequestRulesInterface},
    },
    CoreContext,
};

/// Pull request status.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
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
    /// PR is merged?
    pub merged: bool,
    /// Merge strategy
    pub merge_strategy: MergeStrategy,
    /// Rules
    pub rules: Vec<PullRequestRule>,
}

impl PullRequestStatus {
    /// Create status from pull request and database.
    #[tracing::instrument(skip_all, fields(pr_handle))]
    pub async fn from_database(
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<Self> {
        let fetch_repo_model = ctx.db_service.repositories_get(
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
        );

        let fetch_pr_model = ctx.db_service.pull_requests_get(
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            pr_handle.number(),
        );

        let fetch_upstream_reviews = ReviewApi::list_reviews_for_pull_request(
            ctx.api_service,
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            pr_handle.number(),
        );

        let fetch_required_reviewers = ctx.db_service.required_reviewers_list(
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
            pr_handle.number(),
        );

        let (repo_model, pr_model, upstream_reviews, required_reviewers) = tokio::join!(
            fetch_repo_model,
            fetch_pr_model,
            fetch_upstream_reviews,
            fetch_required_reviewers
        );

        let repo_model = repo_model?.unwrap();
        let pr_model = pr_model?.unwrap();
        let upstream_reviews = upstream_reviews?;
        let required_reviewers = required_reviewers?;

        let checks_status = if pr_model.checks_enabled {
            let determine_check_status: &dyn DetermineChecksStatusInterface =
                ctx.core_module.resolve_ref();
            determine_check_status
                .run(
                    ctx,
                    pr_handle.repository_path(),
                    &upstream_pr.head.sha,
                    pr_model.checks_enabled,
                )
                .await?
        } else {
            ChecksStatus::Skipped
        };

        let strategy = if let Some(s) = pr_model.strategy_override {
            s
        } else {
            let base_branch = &upstream_pr.base.reference;
            let head_branch = &upstream_pr.head.reference;
            let determine_strategy_uc: &dyn DeterminePullRequestMergeStrategyInterface =
                ctx.core_module.resolve_ref();
            determine_strategy_uc
                .run(
                    ctx,
                    pr_handle.repository_path(),
                    head_branch,
                    base_branch,
                    repo_model.default_strategy,
                )
                .await?
        };

        let resolve_rules: &dyn ResolvePullRequestRulesInterface = ctx.core_module.resolve_ref();
        let rules = resolve_rules
            .run(ctx, pr_handle.repository_path(), upstream_pr)
            .await?;

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
            rules,
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

    /// Get rule names.
    pub fn rule_names(&self) -> Option<String> {
        if self.rules.is_empty() {
            None
        } else {
            let names = self
                .rules
                .iter()
                .map(|r| r.name.clone())
                .collect::<Vec<_>>();
            Some(names.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::{
        review::{GhReviewApi, GhReviewStateApi},
        types::{GhBranch, GhUser},
        MockApiService,
    };
    use prbot_models::{PullRequest, Repository};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::checks::MockDetermineChecksStatusInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn blank_no_checks_no_qa_no_reviewers() {
        let mut ctx = CoreContextTest::new();
        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pull_reviews_list()
                .once()
                .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
                .return_once(|_, _, _| Ok(vec![]));

            svc
        };

        ctx.db_service = {
            let svc = MemoryDb::new();

            let repo = svc
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

            svc.pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        let status = PullRequestStatus::from_database(
            &ctx.as_context(),
            &("me", "test", 1).into(),
            &GhPullRequest {
                ..Default::default()
            },
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
                pull_request_title_regex: String::new(),
                rules: vec![]
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_no_qa_no_reviewers() {
        let mut ctx = CoreContextTest::new();
        let determine_checks_status = {
            let mut mock = MockDetermineChecksStatusInterface::new();
            mock.expect_run()
                .once()
                .withf(|_, repository_path, sha, wait_for_checks| {
                    repository_path == &("me", "test").into()
                        && sha == "abcdef"
                        && wait_for_checks == &true
                })
                .return_once(|_, _, _, _| Ok(ChecksStatus::Waiting));

            mock
        };

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pull_reviews_list()
                .once()
                .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
                .return_once(|_, _, _| Ok(vec![]));

            svc
        };

        ctx.db_service = {
            let svc = MemoryDb::new();

            let repo = svc
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

            svc.pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn DetermineChecksStatusInterface>(Box::new(
                determine_checks_status,
            ))
            .build();

        let status = PullRequestStatus::from_database(
            &ctx.as_context(),
            &("me", "test", 1).into(),
            &GhPullRequest {
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
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
                pull_request_title_regex: String::new(),
                rules: vec![]
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_qa_no_reviewers() {
        let mut ctx = CoreContextTest::new();
        let determine_checks_status = {
            let mut mock = MockDetermineChecksStatusInterface::new();
            mock.expect_run()
                .once()
                .withf(|_, repository_path, sha, wait_for_checks| {
                    repository_path == &("me", "test").into()
                        && sha == "abcdef"
                        && wait_for_checks == &true
                })
                .return_once(|_, _, _, _| Ok(ChecksStatus::Waiting));

            mock
        };

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pull_reviews_list()
                .once()
                .withf(|owner, name, pr_number| owner == "me" && name == "test" && pr_number == &1)
                .return_once(|_, _, _| Ok(vec![]));

            svc
        };

        ctx.db_service = {
            let svc = MemoryDb::new();

            let repo = svc
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

            svc.pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn DetermineChecksStatusInterface>(Box::new(
                determine_checks_status,
            ))
            .build();

        let status = PullRequestStatus::from_database(
            &ctx.as_context(),
            &("me", "test", 1).into(),
            &GhPullRequest {
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
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
                pull_request_title_regex: String::new(),
                rules: vec![]
            }
        )
    }

    #[tokio::test]
    async fn blank_checks_qa_reviewers() {
        let mut ctx = CoreContextTest::new();
        let determine_checks_status = {
            let mut mock = MockDetermineChecksStatusInterface::new();
            mock.expect_run()
                .once()
                .withf(|_, repository_path, sha, wait_for_checks| {
                    repository_path == &("me", "test").into()
                        && sha == "abcdef"
                        && wait_for_checks == &true
                })
                .return_once(|_, _, _, _| Ok(ChecksStatus::Waiting));

            mock
        };

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pull_reviews_list()
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

            svc
        };

        ctx.db_service = {
            let svc = MemoryDb::new();

            let repo = svc
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

            svc.pull_requests_create(
                PullRequest {
                    repository_id: repo.id,
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn DetermineChecksStatusInterface>(Box::new(
                determine_checks_status,
            ))
            .build();

        let status = PullRequestStatus::from_database(
            &ctx.as_context(),
            &("me", "test", 1).into(),
            &GhPullRequest {
                head: GhBranch {
                    sha: "abcdef".into(),
                    ..Default::default()
                },
                ..Default::default()
            },
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
                pull_request_title_regex: String::new(),
                rules: vec![]
            }
        )
    }
}
