//! External module.

use async_trait::async_trait;
use prbot_models::{ExternalAccount, QaStatus, RepositoryPath};
use shaku::{Component, HasComponent, Interface};

use crate::{
    bot_commands::{
        commands::{BotCommand, SetQaStatusCommand},
        CommandContext, CommandExecutorInterface,
    },
    CoreContext, Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait SetPullRequestQaStatusInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        external_account: &ExternalAccount,
        repository_path: RepositoryPath,
        pull_request_numbers: &[u64],
        author: &str,
        status: QaStatus,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = SetPullRequestQaStatusInterface)]
pub(crate) struct SetPullRequestQaStatus;

#[async_trait]
impl SetPullRequestQaStatusInterface for SetPullRequestQaStatus {
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
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        external_account: &ExternalAccount,
        repository_path: RepositoryPath,
        pull_request_numbers: &[u64],
        author: &str,
        status: QaStatus,
    ) -> Result<()> {
        let (repo_owner, repo_name) = repository_path.components();
        if ctx
            .db_service
            .external_account_rights_get(repo_owner, repo_name, &external_account.username)
            .await?
            .is_some()
        {
            for pr_number in pull_request_numbers {
                if ctx
                    .db_service
                    .pull_requests_get(repo_owner, repo_name, *pr_number)
                    .await?
                    .is_some()
                {
                    let upstream_pr = ctx
                        .api_service
                        .pulls_get(repo_owner, repo_name, *pr_number)
                        .await?;

                    let ctx = CommandContext {
                        config: ctx.config,
                        core_module: ctx.core_module,
                        api_service: ctx.api_service,
                        db_service: ctx.db_service,
                        lock_service: ctx.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number: *pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: author,
                    };

                    let result = SetQaStatusCommand::new(status).handle(&ctx).await?;
                    let executor: &dyn CommandExecutorInterface = ctx.core_module.resolve_ref();
                    executor.process_command_result(&ctx, &result).await?;
                }
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
        types::{GhBranch, GhPullRequest},
        MockApiService,
    };
    use prbot_models::{ExternalAccountRight, PullRequest, Repository};

    use super::*;
    use crate::{
        context::tests::CoreContextTest,
        use_cases::status::{
            MockUpdatePullRequestStatusInterface, UpdatePullRequestStatusInterface,
        },
        CoreModule,
    };

    #[tokio::test]
    async fn no_rights() {
        let ctx = CoreContextTest::new();

        SetPullRequestQaStatus
            .run(
                &ctx.as_context(),
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
        let mut ctx = CoreContextTest::new();
        ctx.db_service = {
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

        let ext = ctx
            .db_service
            .external_accounts_get("ext")
            .await
            .unwrap()
            .unwrap();

        SetPullRequestQaStatus
            .run(
                &ctx.as_context(),
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
        let mut ctx = CoreContextTest::new();
        let update_pull_request_status = {
            let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();
            update_pull_request_status
                .expect_run()
                .once()
                .withf(|_, pr_handle, upstream_pr| {
                    pr_handle == &("me", "test", 2).into() && upstream_pr.number == 2
                })
                .return_once(|_, _, _| Ok(()));

            update_pull_request_status
        };

        ctx.api_service = {
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

        ctx.db_service = {
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

        let ext = ctx
            .db_service
            .external_accounts_get("ext")
            .await
            .unwrap()
            .unwrap();

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        SetPullRequestQaStatus
            .run(
                &ctx.as_context(),
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
