use async_trait::async_trait;
use prbot_ghapi_interface::types::{GhIssueCommentAction, GhIssueCommentEvent};
use prbot_models::PullRequestHandle;
use shaku::{Component, HasComponent, Interface};
use tracing::info;

use crate::{
    bot_commands::{
        AdminCommand, Command, CommandContext, CommandExecutorInterface, CommandParser,
    },
    use_cases::{pulls::SynchronizePullRequestInterface, status::UpdatePullRequestStatusInterface},
    CoreContext, Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait HandleIssueCommentEventInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhIssueCommentEvent) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = HandleIssueCommentEventInterface)]
pub(crate) struct HandleIssueCommentEvent;

#[async_trait]
impl HandleIssueCommentEventInterface for HandleIssueCommentEvent {
    #[tracing::instrument(skip(self, ctx), fields(
        action = ?event.action,
        repo_owner = event.repository.owner.login,
        repo_name = event.repository.name,
        number = event.issue.number
    ))]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, event: GhIssueCommentEvent) -> Result<()> {
        if let GhIssueCommentAction::Created = event.action {
            self.run_created_comment(ctx, event).await
        } else {
            Ok(())
        }
    }
}

impl HandleIssueCommentEvent {
    async fn run_created_comment(
        &self,
        ctx: &CoreContext<'_>,
        event: GhIssueCommentEvent,
    ) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.issue.number;
        let pr_handle: &PullRequestHandle =
            &(repo_owner.as_str(), repo_name.as_str(), pr_number).into();

        let commands = CommandParser::parse_commands(ctx.config, &event.comment.body);
        match ctx
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_) => {
                let upstream_pr = ctx
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                let ctx = CommandContext {
                    config: ctx.config,
                    api_service: ctx.api_service,
                    db_service: ctx.db_service,
                    lock_service: ctx.lock_service,
                    repo_owner,
                    repo_name,
                    pr_number,
                    upstream_pr: &upstream_pr,
                    comment_id: event.comment.id,
                    comment_author: &event.comment.user.login,
                    core_module: ctx.core_module,
                };

                let command_executor: &dyn CommandExecutorInterface = ctx.core_module.resolve_ref();
                command_executor.execute_commands(&ctx, commands).await?;
            }
            None => {
                // Parse admin enable
                let mut handled = false;
                for command in commands.iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        let upstream_pr = ctx
                            .api_service
                            .pulls_get(repo_owner, repo_name, pr_number)
                            .await?;

                        let synchronize_pull_request: &dyn SynchronizePullRequestInterface =
                            ctx.core_module.resolve_ref();
                        synchronize_pull_request.run(ctx, pr_handle).await?;

                        info!(
                            pull_request_number = event.issue.number,
                            repository_path = %event.repository.full_name,
                            message = "Manual activation on pull request",
                        );

                        let update_pull_request_status: &dyn UpdatePullRequestStatusInterface =
                            ctx.core_module.resolve_ref();
                        update_pull_request_status
                            .run(ctx, pr_handle, &upstream_pr)
                            .await?;

                        handled = true;
                        break;
                    }
                }

                if !handled {
                    info!(
                        commands = ?commands,
                        repository_path = %event.repository.full_name,
                        pull_request_number = event.issue.number,
                        message = "Executing commands on unknown PR",
                    );
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
        types::{GhIssue, GhIssueComment, GhPullRequest, GhRepository, GhUser},
        MockApiService,
    };
    use prbot_models::{PullRequest, Repository};

    use super::*;
    use crate::{
        bot_commands::{
            CommandExecutionResult, CommandHandlingStatus, MockCommandExecutorInterface,
        },
        context::tests::CoreContextTest,
        use_cases::{
            pulls::MockSynchronizePullRequestInterface,
            status::MockUpdatePullRequestStatusInterface,
        },
        CoreModule,
    };

    #[tokio::test]
    async fn run_edited_comment() {
        let ctx = CoreContextTest::new();

        HandleIssueCommentEvent
            .run(
                &ctx.as_context(),
                GhIssueCommentEvent {
                    action: GhIssueCommentAction::Edited,
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    issue: GhIssue {
                        number: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn run_unknown_pr_admin_enable() {
        let mut ctx = CoreContextTest::new();

        ctx.api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pulls_get()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 1,
                        ..Default::default()
                    })
                });

            svc
        };

        let synchronize_pull_request = {
            let mut mock = MockSynchronizePullRequestInterface::new();

            mock.expect_run()
                .once()
                .withf(|_, pr_handle| pr_handle == &("me", "test", 1).into())
                .return_once(|_, _| Ok(()));

            mock
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
            .with_component_override::<dyn SynchronizePullRequestInterface>(Box::new(
                synchronize_pull_request,
            ))
            .build();

        HandleIssueCommentEvent
            .run(
                &ctx.as_context(),
                GhIssueCommentEvent {
                    action: GhIssueCommentAction::Created,
                    comment: GhIssueComment {
                        body: "bot admin-enable".into(),
                        ..Default::default()
                    },
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    issue: GhIssue {
                        number: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn run_unknown_pr_random_command() {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        HandleIssueCommentEvent
            .run(
                &ctx.as_context(),
                GhIssueCommentEvent {
                    action: GhIssueCommentAction::Created,
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    issue: GhIssue {
                        number: 1,
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

        let command_executor = {
            let mut mock = MockCommandExecutorInterface::new();

            mock.expect_execute_commands()
                .once()
                .withf(|_ctx, commands| commands.is_empty())
                .return_once(|_, _| {
                    Ok(CommandExecutionResult {
                        should_update_status: false,
                        handling_status: CommandHandlingStatus::Ignored,
                        result_actions: vec![],
                    })
                });

            mock
        };

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn CommandExecutorInterface>(Box::new(command_executor))
            .build();

        HandleIssueCommentEvent
            .run(
                &ctx.as_context(),
                GhIssueCommentEvent {
                    action: GhIssueCommentAction::Created,
                    repository: GhRepository {
                        owner: GhUser { login: "me".into() },
                        name: "test".into(),
                        ..Default::default()
                    },
                    issue: GhIssue {
                        number: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
}
