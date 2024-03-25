use async_trait::async_trait;
use prbot_ghapi_interface::types::GhCheckSuiteEvent;
use shaku::{Component, HasComponent, Interface};

use crate::{use_cases::status::UpdatePullRequestStatusInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait HandleCheckSuiteEventInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhCheckSuiteEvent) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = HandleCheckSuiteEventInterface)]
pub(crate) struct HandleCheckSuiteEvent;

#[async_trait]
impl HandleCheckSuiteEventInterface for HandleCheckSuiteEvent {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            repository_path = %event.repository.full_name,
            head_branch = %event.check_suite.head_branch,
            head_sha = %event.check_suite.head_sha,
            app_slug = %event.check_suite.app.slug,
            status = ?event.check_suite.status,
            conclusion = ?event.check_suite.conclusion
        )
    )]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhCheckSuiteEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        // Only look for first PR
        if let Some(gh_pr) = event.check_suite.pull_requests.first() {
            let pr_number = gh_pr.number;

            if let Some(pr_model) = ctx
                .db_service
                .pull_requests_get(repo_owner, repo_name, pr_number)
                .await?
            {
                // Skip non Github Actions checks
                if event.check_suite.app.slug != "github-actions" {
                    return Ok(());
                }

                // Skip non up-to-date checks
                if event.check_suite.head_sha != gh_pr.head.sha {
                    return Ok(());
                }

                // Skip if checks are skipped
                if !pr_model.checks_enabled {
                    return Ok(());
                }

                let upstream_pr = ctx
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                // Update status
                let update_pull_request_status: &dyn UpdatePullRequestStatusInterface =
                    ctx.core_module.resolve_ref();
                update_pull_request_status
                    .run(
                        ctx,
                        &(repo_owner.as_str(), repo_name.as_str(), pr_number).into(),
                        &upstream_pr,
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::{
        types::{
            GhApplication, GhBranch, GhBranchShort, GhCheckSuite, GhPullRequest,
            GhPullRequestShort, GhRepository, GhUser,
        },
        MockApiService,
    };
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::status::MockUpdatePullRequestStatusInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn run_unknown_pr() {
        let ctx = CoreContextTest::new();

        HandleCheckSuiteEvent
            .run(
                &ctx.as_context(),
                GhCheckSuiteEvent {
                    check_suite: GhCheckSuite {
                        pull_requests: vec![GhPullRequestShort {
                            number: 1,
                            head: GhBranchShort {
                                sha: "abcdef".into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }],
                        head_sha: "abcdef".into(),
                        app: GhApplication {
                            slug: "github-actions".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn run_known_pr() {
        let mut ctx = CoreContextTest::new();

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pulls_get()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 1,
                        head: GhBranch {
                            sha: "abcdef".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
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

        let update_pull_request_status = {
            let mut mock = MockUpdatePullRequestStatusInterface::new();

            mock.expect_run()
                .once()
                .withf(|_, pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
                })
                .return_once(|_, _, _| Ok(()));

            mock
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        HandleCheckSuiteEvent
            .run(
                &ctx.as_context(),
                GhCheckSuiteEvent {
                    check_suite: GhCheckSuite {
                        pull_requests: vec![GhPullRequestShort {
                            number: 1,
                            head: GhBranchShort {
                                sha: "abcdef".into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }],
                        head_sha: "abcdef".into(),
                        app: GhApplication {
                            slug: "github-actions".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
}
