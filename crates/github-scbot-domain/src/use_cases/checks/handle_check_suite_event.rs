use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::{types::GhCheckSuiteEvent, ApiService};

use crate::{use_cases::status::UpdatePullRequestStatusUseCaseInterface, Result};

pub struct HandleCheckSuiteEventUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

impl<'a> HandleCheckSuiteEventUseCase<'a> {
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
    pub async fn run(&self, event: GhCheckSuiteEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;

        // Only look for first PR
        if let Some(gh_pr) = event.check_suite.pull_requests.get(0) {
            let pr_number = gh_pr.number;

            if let Some(pr_model) = self
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

                let upstream_pr = self
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                // Update status
                self.update_pull_request_status
                    .run(
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
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::{
        types::{
            GhApplication, GhBranch, GhBranchShort, GhCheckSuite, GhPullRequest,
            GhPullRequestShort, GhRepository, GhUser,
        },
        MockApiService,
    };

    use super::*;
    use crate::use_cases::status::MockUpdatePullRequestStatusUseCaseInterface;

    #[tokio::test]
    async fn run_unknown_pr() {
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        HandleCheckSuiteEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhCheckSuiteEvent {
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
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn run_known_pr() {
        let api_service = {
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

        let db_service = {
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
            let mut mock = MockUpdatePullRequestStatusUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
                })
                .return_once(|_, _| Ok(()));

            mock
        };

        HandleCheckSuiteEventUseCase {
            api_service: &api_service,
            db_service: &db_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhCheckSuiteEvent {
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
        })
        .await
        .unwrap();
    }
}
