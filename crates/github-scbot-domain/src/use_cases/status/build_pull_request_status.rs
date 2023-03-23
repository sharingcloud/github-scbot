use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{types::GhPullRequest, ApiService};

use super::utils::PullRequestStatus;
use crate::{use_cases::checks::DetermineChecksStatusUseCaseInterface, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait BuildPullRequestStatusUseCaseInterface {
    async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<PullRequestStatus>;
}

pub struct BuildPullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub determine_checks_status: &'a dyn DetermineChecksStatusUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> BuildPullRequestStatusUseCaseInterface for BuildPullRequestStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle), ret)]
    async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<PullRequestStatus> {
        PullRequestStatus::from_database(
            self.api_service,
            self.db_service,
            pr_handle,
            upstream_pr,
            self.determine_checks_status,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{
        ChecksStatus, MergeStrategy, PullRequest, QaStatus, Repository,
    };
    use github_scbot_ghapi_interface::{types::GhBranch, MockApiService};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::use_cases::checks::MockDetermineChecksStatusUseCaseInterface;

    #[tokio::test]
    async fn run() {
        let determine_checks_status = {
            let mut svc = MockDetermineChecksStatusUseCaseInterface::new();

            svc.expect_run()
                .once()
                .withf(|repository_path, sha, wait_for_checks| {
                    repository_path == &("me", "test").into()
                        && sha == "abcdef"
                        && wait_for_checks == &true
                })
                .return_once(|_, _, _| Ok(ChecksStatus::Waiting));

            svc
        };

        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_pull_reviews_list()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| Ok(vec![]));

            svc
        };

        let db_service = {
            let svc = MemoryDb::new();

            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    default_enable_checks: true,
                    default_needed_reviewers_count: 1,
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

        let status = BuildPullRequestStatusUseCase {
            api_service: &api_service,
            db_service: &db_service,
            determine_checks_status: &determine_checks_status,
        }
        .run(
            &("me", "test", 1).into(),
            &GhPullRequest {
                number: 1,
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
                changes_required_reviewers: vec![],
                approved_reviewers: vec![],
                automerge: false,
                checks_status: ChecksStatus::Waiting,
                checks_url: "https://github.com/me/test/pull/1/checks".into(),
                qa_status: QaStatus::Skipped,
                needed_reviewers_count: 1,
                missing_required_reviewers: vec![],
                pull_request_title_regex: String::new(),
                valid_pr_title: true,
                locked: false,
                mergeable: true,
                wip: false,
                merged: false,
                merge_strategy: MergeStrategy::Merge
            }
        );
    }
}
