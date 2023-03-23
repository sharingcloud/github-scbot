//! External module.

use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{ExternalAccount, QaStatus, RepositoryPath};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::UpdatePullRequestStatusUseCaseInterface;
use crate::{
    commands::{
        commands::{BotCommand, SetQaStatusCommand},
        CommandContext, CommandExecutor, CommandExecutorInterface,
    },
    Result,
};

/// Set QA status for multiple pull request numbers.
pub struct SetPullRequestQaStatusUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

impl<'a> SetPullRequestQaStatusUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            external_account = external_account.username,
            repository_path = %repository_path,
            pr_numbers = ?pull_request_numbers,
            author = %author,
            status = ?status
        )
    )]
    pub async fn run(
        &self,
        external_account: &ExternalAccount,
        repository_path: RepositoryPath,
        pull_request_numbers: &[u64],
        author: &str,
        status: QaStatus,
    ) -> Result<()> {
        let (repo_owner, repo_name) = repository_path.components();
        if self
            .db_service
            .external_account_rights_get(repo_owner, repo_name, &external_account.username)
            .await?
            .is_some()
        {
            for pr_number in pull_request_numbers {
                if self
                    .db_service
                    .pull_requests_get(repo_owner, repo_name, *pr_number)
                    .await?
                    .is_some()
                {
                    let upstream_pr = self
                        .api_service
                        .pulls_get(repo_owner, repo_name, *pr_number)
                        .await?;

                    let ctx = CommandContext {
                        config: self.config,
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number: *pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: author,
                    };

                    let result = SetQaStatusCommand::new(status).handle(&ctx).await?;
                    let executor = CommandExecutor {
                        db_service: self.db_service,
                        update_pull_request_status: self.update_pull_request_status,
                    };
                    executor.process_command_result(&ctx, &result).await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{ExternalAccountRight, PullRequest, Repository};
    use github_scbot_ghapi_interface::{
        types::{GhBranch, GhPullRequest},
        MockApiService,
    };
    use github_scbot_lock_interface::MockLockService;

    use super::*;
    use crate::use_cases::status::MockUpdatePullRequestStatusUseCaseInterface;

    #[tokio::test]
    async fn no_rights() {
        let config = Config::from_env();
        let lock_service = MockLockService::new();
        let db_service = MemoryDb::new();
        let api_service = MockApiService::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        SetPullRequestQaStatusUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(
            &ExternalAccount {
                username: "ext".into(),
                ..Default::default()
            },
            ("me", "test").into(),
            &[1],
            "author",
            QaStatus::Pass,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn rights_but_unknown_pr() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let lock_service = MockLockService::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        let db_service = {
            let db_service = MemoryDb::new();

            let repo = db_service
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            db_service
                .external_accounts_create(ExternalAccount {
                    username: "ext".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            db_service
                .external_account_rights_create(ExternalAccountRight {
                    repository_id: repo.id,
                    username: "ext".into(),
                })
                .await
                .unwrap();

            db_service
        };

        let ext = db_service
            .external_accounts_get("ext")
            .await
            .unwrap()
            .unwrap();

        SetPullRequestQaStatusUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(
            &ext,
            ("me", "test").into(),
            &[1, 2, 3],
            "author",
            QaStatus::Pass,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn rights_with_known_and_unknown_pr() {
        let config = Config::from_env();
        let update_pull_request_status = {
            let mut update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
            update_pull_request_status
                .expect_run()
                .once()
                .withf(|pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 2).into() && upstream_pr.number == 2
                })
                .return_once(|_, _| Ok(()));

            update_pull_request_status
        };

        let api_service = {
            let mut api_service = MockApiService::new();
            api_service
                .expect_pulls_get()
                .times(2)
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &2)
                .returning(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 2,
                        head: GhBranch {
                            sha: "abcdef".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                });

            api_service
                .expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me" && name == "test" && number == &2 && !body.is_empty()
                })
                .return_once(|_, _, _, _| Ok(1));

            api_service
        };
        let lock_service = MockLockService::new();
        let db_service = {
            let db_service = MemoryDb::new();

            let repo = db_service
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            db_service
                .pull_requests_create(
                    PullRequest {
                        number: 2,
                        ..Default::default()
                    }
                    .with_repository(&repo),
                )
                .await
                .unwrap();
            db_service
                .external_accounts_create(ExternalAccount {
                    username: "ext".into(),
                    ..Default::default()
                })
                .await
                .unwrap();
            db_service
                .external_account_rights_create(ExternalAccountRight {
                    repository_id: repo.id,
                    username: "ext".into(),
                })
                .await
                .unwrap();

            db_service
        };

        let ext = db_service
            .external_accounts_get("ext")
            .await
            .unwrap()
            .unwrap();

        SetPullRequestQaStatusUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(
            &ext,
            ("me", "test").into(),
            &[1, 2, 3],
            "author",
            QaStatus::Pass,
        )
        .await
        .unwrap();
    }
}
