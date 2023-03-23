use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{
    types::{GhIssueCommentAction, GhIssueCommentEvent},
    ApiService,
};
use github_scbot_lock_interface::LockService;
use tracing::info;

use crate::{
    commands::{AdminCommand, Command, CommandContext, CommandExecutorInterface, CommandParser},
    use_cases::{
        pulls::SynchronizePullRequestUseCaseInterface,
        status::UpdatePullRequestStatusUseCaseInterface,
    },
    Result,
};

pub struct HandleIssueCommentEventUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub synchronize_pull_request: &'a dyn SynchronizePullRequestUseCaseInterface,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
    pub command_executor: &'a dyn CommandExecutorInterface,
}

impl<'a> HandleIssueCommentEventUseCase<'a> {
    #[tracing::instrument(skip(self), fields(
        action = ?event.action,
        repo_owner = event.repository.owner.login,
        repo_name = event.repository.name,
        number = event.issue.number
    ))]
    pub async fn run(&self, event: GhIssueCommentEvent) -> Result<()> {
        if let GhIssueCommentAction::Created = event.action {
            self.run_created_comment(event).await
        } else {
            Ok(())
        }
    }

    async fn run_created_comment(&self, event: GhIssueCommentEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.issue.number;
        let pr_handle: &PullRequestHandle =
            &(repo_owner.as_str(), repo_name.as_str(), pr_number).into();

        let commands = CommandParser::parse_commands(self.config, &event.comment.body);
        match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_) => {
                let upstream_pr = self
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                let ctx = CommandContext {
                    config: self.config,
                    api_service: self.api_service,
                    db_service: self.db_service,
                    lock_service: self.lock_service,
                    repo_owner,
                    repo_name,
                    pr_number,
                    upstream_pr: &upstream_pr,
                    comment_id: event.comment.id,
                    comment_author: &event.comment.user.login,
                };

                self.command_executor
                    .execute_commands(&ctx, commands)
                    .await?;
            }
            None => {
                // Parse admin enable
                let mut handled = false;
                for command in commands.iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        let upstream_pr = self
                            .api_service
                            .pulls_get(repo_owner, repo_name, pr_number)
                            .await?;

                        self.synchronize_pull_request.run(pr_handle).await?;

                        info!(
                            pull_request_number = event.issue.number,
                            repository_path = %event.repository.full_name,
                            message = "Manual activation on pull request",
                        );

                        self.update_pull_request_status
                            .run(pr_handle, &upstream_pr)
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
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::{
        types::{GhIssue, GhIssueComment, GhPullRequest, GhRepository, GhUser},
        MockApiService,
    };
    use github_scbot_lock_interface::MockLockService;

    use super::*;
    use crate::{
        commands::{CommandExecutionResult, CommandHandlingStatus, MockCommandExecutorInterface},
        use_cases::{
            pulls::MockSynchronizePullRequestUseCaseInterface,
            status::MockUpdatePullRequestStatusUseCaseInterface,
        },
    };

    #[tokio::test]
    async fn run_edited_comment() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let lock_service = MockLockService::new();
        let db_service = MemoryDb::new();
        let synchronize_pull_request = MockSynchronizePullRequestUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        let command_executor = MockCommandExecutorInterface::new();

        HandleIssueCommentEventUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            synchronize_pull_request: &synchronize_pull_request,
            update_pull_request_status: &update_pull_request_status,
            command_executor: &command_executor,
        }
        .run(GhIssueCommentEvent {
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
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn run_unknown_pr_admin_enable() {
        let config = Config::from_env();
        let lock_service = MockLockService::new();
        let db_service = MemoryDb::new();
        let command_executor = MockCommandExecutorInterface::new();

        let api_service = {
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
            let mut mock = MockSynchronizePullRequestUseCaseInterface::new();

            mock.expect_run()
                .once()
                .withf(|pr_handle| pr_handle == &("me", "test", 1).into())
                .return_once(|_| Ok(()));

            mock
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

        HandleIssueCommentEventUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            synchronize_pull_request: &synchronize_pull_request,
            update_pull_request_status: &update_pull_request_status,
            command_executor: &command_executor,
        }
        .run(GhIssueCommentEvent {
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
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn run_unknown_pr_random_command() {
        let config = Config::from_env();
        let lock_service = MockLockService::new();
        let synchronize_pull_request = MockSynchronizePullRequestUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        let command_executor = MockCommandExecutorInterface::new();
        let api_service = MockApiService::new();

        let db_service = {
            let svc = MemoryDb::new();

            svc.repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

            svc
        };

        HandleIssueCommentEventUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            synchronize_pull_request: &synchronize_pull_request,
            update_pull_request_status: &update_pull_request_status,
            command_executor: &command_executor,
        }
        .run(GhIssueCommentEvent {
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
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn run_known_pr() {
        let config = Config::from_env();
        let lock_service = MockLockService::new();
        let synchronize_pull_request = MockSynchronizePullRequestUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        let api_service = {
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

        let db_service = {
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

        HandleIssueCommentEventUseCase {
            config: &config,
            api_service: &api_service,
            db_service: &db_service,
            lock_service: &lock_service,
            synchronize_pull_request: &synchronize_pull_request,
            update_pull_request_status: &update_pull_request_status,
            command_executor: &command_executor,
        }
        .run(GhIssueCommentEvent {
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
        })
        .await
        .unwrap();
    }
}
