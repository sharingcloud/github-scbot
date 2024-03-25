use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequestEvent;
use prbot_models::{PullRequest, PullRequestHandle, Repository};
use shaku::{Component, HasComponent, Interface};

use crate::{
    bot_commands::{
        AdminCommand, Command, CommandContext, CommandExecutorInterface, CommandParser,
    },
    use_cases::{
        comments::PostWelcomeCommentInterface,
        pulls::{
            ApplyPullRequestRulesInterface, GetOrCreateRepositoryInterface,
            ResolvePullRequestRulesInterface,
        },
        status::UpdatePullRequestStatusInterface,
    },
    CoreContext, Result,
};

/// Pull request opened status.
#[derive(Debug, PartialEq, Eq)]
pub enum PullRequestOpenedStatus {
    /// Pull request is already created.
    AlreadyCreated,
    /// Pull request is created.
    Created,
    /// Pull request is ignored.
    Ignored,
}

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait ProcessPullRequestOpenedInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        event: GhPullRequestEvent,
    ) -> Result<PullRequestOpenedStatus>;
}

#[derive(Component)]
#[shaku(interface = ProcessPullRequestOpenedInterface)]
pub(crate) struct ProcessPullRequestOpened;

#[async_trait]
impl ProcessPullRequestOpenedInterface for ProcessPullRequestOpened {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?event.action,
            pr_number = event.number,
            repository_path = %event.repository.full_name,
            username = %event.pull_request.user.login
        ),
        ret
    )]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        event: GhPullRequestEvent,
    ) -> Result<PullRequestOpenedStatus> {
        // Get or create repository
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.pull_request.number;
        let pr_handle: &PullRequestHandle =
            &(repo_owner.as_str(), repo_name.as_str(), pr_number).into();

        let get_or_create_uc: &dyn GetOrCreateRepositoryInterface = ctx.core_module.resolve_ref();
        let repo_model = get_or_create_uc
            .run(ctx, pr_handle.repository_path())
            .await?;

        match ctx
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
            None => {
                if Self::should_create_pull_request(ctx, &repo_model, &event) {
                    let pr_model = ctx
                        .db_service
                        .pull_requests_create(
                            PullRequest {
                                number: event.pull_request.number,
                                ..Default::default()
                            }
                            .with_repository(&repo_model),
                        )
                        .await?;

                    // Get upstream pull request
                    let upstream_pr = ctx
                        .api_service
                        .pulls_get(&repo_model.owner, &repo_model.name, pr_model.number)
                        .await?;

                    let update_pull_request_status: &dyn UpdatePullRequestStatusInterface =
                        ctx.core_module.resolve_ref();
                    update_pull_request_status
                        .run(ctx, pr_handle, &upstream_pr)
                        .await?;

                    if ctx.config.server.enable_welcome_comments {
                        let post_welcome_comment: &dyn PostWelcomeCommentInterface =
                            ctx.core_module.resolve_ref();
                        post_welcome_comment
                            .run(ctx, pr_handle, &event.pull_request.user.login)
                            .await?;
                    }

                    // Now, handle commands from body.
                    let commands = CommandParser::parse_commands(
                        ctx.config,
                        event.pull_request.body.as_deref().unwrap_or_default(),
                    );

                    let command_ctx = CommandContext {
                        config: ctx.config,
                        core_module: ctx.core_module,
                        api_service: ctx.api_service,
                        db_service: ctx.db_service,
                        lock_service: ctx.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: &event.pull_request.user.login,
                    };

                    let executor: &dyn CommandExecutorInterface = ctx.core_module.resolve_ref();
                    executor.execute_commands(&command_ctx, commands).await?;

                    // Resolve and apply rules for this pull request
                    let resolve_rules: &dyn ResolvePullRequestRulesInterface =
                        ctx.core_module.resolve_ref();
                    let rules = resolve_rules
                        .run(ctx, pr_handle.repository_path(), &upstream_pr)
                        .await?;

                    let apply_rules: &dyn ApplyPullRequestRulesInterface =
                        ctx.core_module.resolve_ref();
                    apply_rules.run(ctx, pr_handle, rules).await?;

                    Ok(PullRequestOpenedStatus::Created)
                } else {
                    Ok(PullRequestOpenedStatus::Ignored)
                }
            }
        }
    }
}

impl ProcessPullRequestOpened {
    pub fn should_create_pull_request(
        ctx: &CoreContext<'_>,
        repo_model: &Repository,
        event: &GhPullRequestEvent,
    ) -> bool {
        if repo_model.manual_interaction {
            if let Some(body) = &event.pull_request.body {
                // Check for magic instruction to enable bot
                let commands = CommandParser::parse_commands(ctx.config, body);
                for command in commands.into_iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        return true;
                    }
                }
            }

            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::types::{GhPullRequest, GhRepository, GhUser, GhUserPermission};

    use super::*;
    use crate::{
        context::tests::CoreContextTest, use_cases::status::MockUpdatePullRequestStatusInterface,
        CoreModule,
    };

    #[tokio::test]
    async fn no_manual_interaction() {
        let mut ctx = CoreContextTest::new();
        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: false,
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|_, pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _, _| Ok(()));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        let result = ProcessPullRequestOpened
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    pull_request: GhPullRequest {
                        number: 1,
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
            .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }

    #[tokio::test]
    async fn already_created() {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpened
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    pull_request: GhPullRequest {
                        number: 1,
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
            .await;

        assert!(matches!(
            result,
            Ok(PullRequestOpenedStatus::AlreadyCreated)
        ))
    }

    #[tokio::test]
    async fn manual_interaction_without_comment() {
        let ctx = CoreContextTest::new();

        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpened
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    pull_request: GhPullRequest {
                        number: 1,
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
            .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_wrong_comment() {
        let ctx = CoreContextTest::new();
        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpened
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    pull_request: GhPullRequest {
                        number: 1,
                        body: Some("bot hello".into()),
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
            .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_enable_comment_non_admin_user() {
        let mut ctx = CoreContextTest::new();
        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        ctx.api_service
            .expect_user_permissions_get()
            .once()
            .withf(|owner, name, username| owner == "me" && name == "test" && username == "user")
            .return_once(|_, _, _| Ok(GhUserPermission::Write));

        let mut update_pull_request_status = MockUpdatePullRequestStatusInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|_, pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _, _| Ok(()));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn UpdatePullRequestStatusInterface>(Box::new(
                update_pull_request_status,
            ))
            .build();

        let result = ProcessPullRequestOpened
            .run(
                &ctx.as_context(),
                GhPullRequestEvent {
                    pull_request: GhPullRequest {
                        number: 1,
                        body: Some("bot admin-enable".into()),
                        user: GhUser {
                            login: "user".into(),
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
            .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }
}
