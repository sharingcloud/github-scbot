use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequest;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use super::utils::PullRequestStatus;
use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait BuildPullRequestStatusInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<PullRequestStatus>;
}

#[derive(Component)]
#[shaku(interface = BuildPullRequestStatusInterface)]
pub(crate) struct BuildPullRequestStatus;

#[async_trait]
impl BuildPullRequestStatusInterface for BuildPullRequestStatus {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle), ret)]
    async fn run<'b>(
        &self,
        ctx: &CoreContext<'b>,
        pr_handle: &PullRequestHandle,
        upstream_pr: &GhPullRequest,
    ) -> Result<PullRequestStatus> {
        PullRequestStatus::from_database(ctx, pr_handle, upstream_pr).await
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::{types::GhBranch, MockApiService};
    use prbot_models::{ChecksStatus, MergeStrategy, PullRequest, QaStatus, Repository};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::checks::{DetermineChecksStatusInterface, MockDetermineChecksStatusInterface},
        CoreModule,
    };

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();

        let determine_checks_status = {
            let mut svc = MockDetermineChecksStatusInterface::new();

            svc.expect_run()
                .once()
                .withf(|_, repository_path, sha, wait_for_checks| {
                    repository_path == &("me", "test").into()
                        && sha == "abcdef"
                        && wait_for_checks == &true
                })
                .return_once(|_, _, _, _| Ok(ChecksStatus::Waiting));

            svc
        };

        ctx.api_service = {
            let mut svc = MockApiService::new();
            svc.expect_pull_reviews_list()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
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

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn DetermineChecksStatusInterface>(Box::new(
                determine_checks_status,
            ))
            .build();

        let status = BuildPullRequestStatus
            .run(
                &ctx.as_context(),
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
                merge_strategy: MergeStrategy::Merge,
                rules: vec![]
            }
        );
    }
}
