use async_trait::async_trait;
use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequest, PullRequestHandle, Repository};
use github_scbot_ghapi_interface::{types::GhPullRequestEvent, ApiService};
use github_scbot_lock_interface::LockService;

use crate::{
    commands::{
        AdminCommand, Command, CommandContext, CommandExecutor, CommandExecutorInterface,
        CommandParser,
    },
    use_cases::{
        comments::PostWelcomeCommentUseCaseInterface, pulls::GetOrCreateRepositoryUseCase,
        status::UpdatePullRequestStatusUseCaseInterface,
    },
    Result,
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

#[async_trait(?Send)]
pub trait ProcessPullRequestOpenedUseCaseInterface {
    async fn run(&self, event: GhPullRequestEvent) -> Result<PullRequestOpenedStatus>;
}

pub struct ProcessPullRequestOpenedUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub post_welcome_comment: &'a dyn PostWelcomeCommentUseCaseInterface,
    pub update_pull_request_status: &'a dyn UpdatePullRequestStatusUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> ProcessPullRequestOpenedUseCaseInterface for ProcessPullRequestOpenedUseCase<'a> {
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
    async fn run(&self, event: GhPullRequestEvent) -> Result<PullRequestOpenedStatus> {
        // Get or create repository
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.pull_request.number;
        let pr_handle: &PullRequestHandle =
            &(repo_owner.as_str(), repo_name.as_str(), pr_number).into();

        let repo_model = GetOrCreateRepositoryUseCase {
            config: self.config,
            db_service: self.db_service,
        }
        .run(pr_handle.repository())
        .await?;

        match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
            None => {
                if Self::should_create_pull_request(self.config, &repo_model, &event) {
                    let pr_model = self
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
                    let upstream_pr = self
                        .api_service
                        .pulls_get(&repo_model.owner, &repo_model.name, pr_model.number)
                        .await?;

                    self.update_pull_request_status
                        .run(pr_handle, &upstream_pr)
                        .await?;

                    if self.config.server_enable_welcome_comments {
                        self.post_welcome_comment
                            .run(pr_handle, &event.pull_request.user.login)
                            .await?;
                    }

                    // Now, handle commands from body.
                    let commands = CommandParser::parse_commands(
                        self.config,
                        event.pull_request.body.as_deref().unwrap_or_default(),
                    );

                    let ctx = CommandContext {
                        config: self.config,
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: &event.pull_request.user.login,
                    };

                    let executor = CommandExecutor {
                        db_service: self.db_service,
                        update_pull_request_status: self.update_pull_request_status,
                    };
                    executor.execute_commands(&ctx, commands).await?;

                    Ok(PullRequestOpenedStatus::Created)
                } else {
                    Ok(PullRequestOpenedStatus::Ignored)
                }
            }
        }
    }
}

impl<'a> ProcessPullRequestOpenedUseCase<'a> {
    pub fn should_create_pull_request(
        config: &Config,
        repo_model: &Repository,
        event: &GhPullRequestEvent,
    ) -> bool {
        if repo_model.manual_interaction {
            if let Some(body) = &event.pull_request.body {
                // Check for magic instruction to enable bot
                let commands = CommandParser::parse_commands(config, body);
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
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_ghapi_interface::{
        types::{GhPullRequest, GhRepository, GhUser, GhUserPermission},
        MockApiService,
    };
    use github_scbot_lock_interface::MockLockService;

    use super::*;
    use crate::use_cases::{
        comments::MockPostWelcomeCommentUseCaseInterface,
        status::MockUpdatePullRequestStatusUseCaseInterface,
    };

    #[tokio::test]
    async fn no_manual_interaction() {
        let config = Config::from_env();
        let lock_service = MockLockService::new();
        let post_welcome_comment = MockPostWelcomeCommentUseCaseInterface::new();

        let db_service = MemoryDb::new();
        db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: false,
                ..Default::default()
            })
            .await
            .unwrap();

        let mut api_service = MockApiService::new();
        api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        let mut update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _| Ok(()));

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &db_service,
            lock_service: &lock_service,
            post_welcome_comment: &post_welcome_comment,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }

    #[tokio::test]
    async fn already_created() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let lock_service = MockLockService::new();
        let post_welcome_comment = MockPostWelcomeCommentUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

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
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &db_service,
            lock_service: &lock_service,
            post_welcome_comment: &post_welcome_comment,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await;

        assert!(matches!(
            result,
            Ok(PullRequestOpenedStatus::AlreadyCreated)
        ))
    }

    #[tokio::test]
    async fn manual_interaction_without_comment() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let lock_service = MockLockService::new();
        let post_welcome_comment = MockPostWelcomeCommentUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &db_service,
            lock_service: &lock_service,
            post_welcome_comment: &post_welcome_comment,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_wrong_comment() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let lock_service = MockLockService::new();
        let post_welcome_comment = MockPostWelcomeCommentUseCaseInterface::new();
        let update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();

        db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &db_service,
            lock_service: &lock_service,
            post_welcome_comment: &post_welcome_comment,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_enable_comment_non_admin_user() {
        let config = Config::from_env();
        let mut api_service = MockApiService::new();
        let db_service = MemoryDb::new();
        let lock_service = MockLockService::new();
        let post_welcome_comment = MockPostWelcomeCommentUseCaseInterface::new();

        db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                manual_interaction: true,
                ..Default::default()
            })
            .await
            .unwrap();

        api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    number: 1,
                    ..Default::default()
                })
            });

        api_service
            .expect_user_permissions_get()
            .once()
            .withf(|owner, name, username| owner == "me" && name == "test" && username == "user")
            .return_once(|_, _, _| Ok(GhUserPermission::Write));

        let mut update_pull_request_status = MockUpdatePullRequestStatusUseCaseInterface::new();
        update_pull_request_status
            .expect_run()
            .once()
            .withf(|pr_handle, upstream_pr| {
                pr_handle == &("me", "test", 1).into() && upstream_pr.number == 1
            })
            .return_once(|_, _| Ok(()));

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &db_service,
            lock_service: &lock_service,
            post_welcome_comment: &post_welcome_comment,
            update_pull_request_status: &update_pull_request_status,
        }
        .run(GhPullRequestEvent {
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
        })
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }
}
