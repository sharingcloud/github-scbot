use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequest, Repository};
use github_scbot_ghapi_interface::{types::GhPullRequestEvent, ApiService};
use github_scbot_lock_interface::LockService;

use super::GetOrCreateRepositoryUseCase;
use crate::{
    commands::{AdminCommand, Command, CommandContext, CommandExecutor, CommandParser},
    use_cases::{comments::PostWelcomeCommentUseCase, status::UpdatePullRequestStatusUseCase},
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

pub struct ProcessPullRequestOpenedUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub event: GhPullRequestEvent,
}

impl<'a> ProcessPullRequestOpenedUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?self.event.action,
            pr_number = self.event.number,
            repository_path = %self.event.repository.full_name,
            username = %self.event.pull_request.user.login
        )
    )]
    pub async fn run(&mut self) -> Result<PullRequestOpenedStatus> {
        // Get or create repository
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;
        let pr_number = self.event.pull_request.number;

        let repo_model = GetOrCreateRepositoryUseCase {
            db_service: self.db_service,
            config: self.config,
            repo_name,
            repo_owner,
        }
        .run()
        .await?;

        match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
            None => {
                if Self::should_create_pull_request(self.config, &repo_model, &self.event) {
                    let pr_model = self
                        .db_service
                        .pull_requests_create(
                            PullRequest {
                                number: self.event.pull_request.number,
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

                    UpdatePullRequestStatusUseCase {
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_name,
                        repo_owner,
                        pr_number,
                        upstream_pr: &upstream_pr,
                    }
                    .run()
                    .await?;

                    if self.config.server_enable_welcome_comments {
                        PostWelcomeCommentUseCase {
                            api_service: self.api_service,
                            repo_owner,
                            repo_name,
                            pr_number,
                            pr_author: &self.event.pull_request.user.login,
                        }
                        .run()
                        .await?;
                    }

                    // Now, handle commands from body.
                    let commands = CommandParser::parse_commands(
                        self.config,
                        self.event.pull_request.body.as_deref().unwrap_or_default(),
                    );

                    let mut ctx = CommandContext {
                        config: self.config,
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: &self.event.pull_request.user.login,
                    };

                    CommandExecutor::execute_commands(&mut ctx, commands).await?;

                    Ok(PullRequestOpenedStatus::Created)
                } else {
                    Ok(PullRequestOpenedStatus::Ignored)
                }
            }
        }
    }

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
        types::{GhBranch, GhCommitStatus, GhPullRequest, GhRepository, GhUser, GhUserPermission},
        MockApiService,
    };
    use github_scbot_lock_interface::{LockInstance, LockStatus, MockLockService};

    use super::*;

    fn prepare_lock_service_calls() -> MockLockService {
        let mut lock_service = MockLockService::new();

        lock_service
            .expect_wait_lock_resource()
            .once()
            .withf(|name, timeout| name == "summary-me-test-1" && timeout == &10000)
            .return_once(|_, _| {
                Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                    "dummy",
                )))
            });

        lock_service
    }

    fn prepare_api_service_calls() -> MockApiService {
        let mut api_service = MockApiService::new();

        api_service
            .expect_pulls_get()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| {
                Ok(GhPullRequest {
                    head: GhBranch {
                        sha: "abcdef".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            });

        api_service
            .expect_pull_reviews_list()
            .once()
            .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_check_runs_list()
            .once()
            .withf(|owner, name, sha| owner == "me" && name == "test" && sha == "abcdef")
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, issue_id| owner == "me" && name == "test" && issue_id == &1)
            .return_once(|_, _, _| Ok(vec![]));

        api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, issue_id, labels| {
                owner == "me"
                    && name == "test"
                    && issue_id == &1
                    && labels == ["step/awaiting-checks".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        api_service
            .expect_comments_post()
            .once()
            .withf(|owner, name, pr_id, text| {
                owner == "me" && name == "test" && pr_id == &1 && !text.is_empty()
            })
            .return_once(|_, _, _, _| Ok(1));

        api_service
            .expect_commit_statuses_update()
            .once()
            .withf(|owner, name, sha, status, title, body| {
                owner == "me"
                    && name == "test"
                    && sha == "abcdef"
                    && *status == GhCommitStatus::Pending
                    && title == "Validation"
                    && body == "Waiting for checks"
            })
            .return_once(|_, _, _, _, _, _| Ok(()));

        api_service
    }

    #[tokio::test]
    async fn no_manual_interaction() {
        let config = Config::from_env();
        let api_service = prepare_api_service_calls();
        let mut db_service = MemoryDb::new();
        let lock_service = prepare_lock_service_calls();

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &mut db_service,
            event: GhPullRequestEvent {
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
            lock_service: &lock_service,
        }
        .run()
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }

    #[tokio::test]
    async fn already_created() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let mut db_service = MemoryDb::new();
        let lock_service = MockLockService::new();

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
            db_service: &mut db_service,
            event: GhPullRequestEvent {
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
            lock_service: &lock_service,
        }
        .run()
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
        let mut db_service = MemoryDb::new();
        let lock_service = MockLockService::new();

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
            db_service: &mut db_service,
            event: GhPullRequestEvent {
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
            lock_service: &lock_service,
        }
        .run()
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_wrong_comment() {
        let config = Config::from_env();
        let api_service = MockApiService::new();
        let mut db_service = MemoryDb::new();
        let lock_service = MockLockService::new();

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
            db_service: &mut db_service,
            event: GhPullRequestEvent {
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
            lock_service: &lock_service,
        }
        .run()
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Ignored)))
    }

    #[tokio::test]
    async fn manual_interaction_with_enable_comment_non_admin_user() {
        let config = Config::from_env();
        let mut api_service = prepare_api_service_calls();
        let mut db_service = MemoryDb::new();
        let lock_service = prepare_lock_service_calls();

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
            .expect_user_permissions_get()
            .once()
            .withf(|owner, name, username| owner == "me" && name == "test" && username == "user")
            .return_once(|_, _, _| Ok(GhUserPermission::Write));

        let result = ProcessPullRequestOpenedUseCase {
            api_service: &api_service,
            config: &config,
            db_service: &mut db_service,
            event: GhPullRequestEvent {
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
            lock_service: &lock_service,
        }
        .run()
        .await;

        assert!(matches!(result, Ok(PullRequestOpenedStatus::Created)))
    }
}
