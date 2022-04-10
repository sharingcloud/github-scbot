use github_scbot_database::{
    models::{DatabaseAdapter, IDatabaseAdapter, RepositoryModel},
    tests::using_test_db,
    Result as DatabaseResult,
};
use github_scbot_ghapi::adapter::DummyAPIAdapter;
use github_scbot_redis::{DummyRedisAdapter, LockInstance, LockStatus};
use github_scbot_types::{
    common::{GhRepository, GhUser},
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent},
    status::{CheckStatus, QaStatus},
};

use crate::{
    pulls::{handle_pull_request_opened, PullRequestLogic, PullRequestOpenedStatus},
    status::PullRequestStatus,
    LogicError,
};

fn fake_gh_repo() -> GhRepository {
    GhRepository {
        name: "name".to_string(),
        owner: GhUser {
            login: "owner".to_string(),
        },
        full_name: "owner/name".to_string(),
    }
}

fn fake_gh_pr() -> GhPullRequest {
    GhPullRequest {
        number: 1,
        ..Default::default()
    }
}

#[actix_rt::test]
async fn test_should_create_pull_request_manual_no_activation() -> DatabaseResult<()> {
    using_test_db(
        "test_db_pr_creation_no_activation",
        |config, pool| async move {
            let db_adapter = DatabaseAdapter::new(pool);

            let creation_event = GhPullRequestEvent {
                action: GhPullRequestAction::Opened,
                repository: GhRepository {
                    name: "name".to_string(),
                    owner: GhUser {
                        login: "owner".to_string(),
                    },
                    full_name: "owner/name".to_string(),
                },
                pull_request: GhPullRequest {
                    number: 1,
                    ..GhPullRequest::default()
                },
                ..GhPullRequestEvent::default()
            };

            let repository =
                RepositoryModel::builder_from_github(&config, &creation_event.repository)
                    .manual_interaction(true)
                    .create_or_update(db_adapter.repository())
                    .await?;

            // Manual interaction without activation
            assert!(!PullRequestLogic::should_create_pull_request(
                &config,
                &repository,
                &creation_event
            ));

            Ok::<_, LogicError>(())
        },
    )
    .await
}

#[actix_rt::test]
async fn test_should_create_pull_request_manual_with_activation() -> DatabaseResult<()> {
    using_test_db(
        "test_db_pr_creation_activation",
        |config, pool| async move {
            let db_adapter = DatabaseAdapter::new(pool);
            let creation_event = GhPullRequestEvent {
                action: GhPullRequestAction::Opened,
                repository: GhRepository {
                    name: "name".to_string(),
                    owner: GhUser {
                        login: "owner".to_string(),
                    },
                    full_name: "owner/name".to_string(),
                },
                pull_request: GhPullRequest {
                    number: 1,
                    body: Some("Hello.\ntest-bot admin-enable".to_string()),
                    ..GhPullRequest::default()
                },
                ..GhPullRequestEvent::default()
            };

            let repository =
                RepositoryModel::builder_from_github(&config, &creation_event.repository)
                    .manual_interaction(true)
                    .create_or_update(db_adapter.repository())
                    .await?;

            // Manual interaction with activation
            assert!(PullRequestLogic::should_create_pull_request(
                &config,
                &repository,
                &creation_event
            ));
            Ok::<_, LogicError>(())
        },
    )
    .await
}

#[actix_rt::test]
async fn test_should_create_pull_request_automatic() -> DatabaseResult<()> {
    using_test_db("test_db_pr_creation_automatic", |config, pool| async move {
        let db_adapter = DatabaseAdapter::new(pool);
        let creation_event = GhPullRequestEvent {
            action: GhPullRequestAction::Opened,
            repository: GhRepository {
                name: "name".to_string(),
                owner: GhUser {
                    login: "owner".to_string(),
                },
                full_name: "owner/name".to_string(),
            },
            pull_request: GhPullRequest {
                number: 1,
                ..GhPullRequest::default()
            },
            ..GhPullRequestEvent::default()
        };

        let repository = RepositoryModel::builder_from_github(&config, &creation_event.repository)
            .manual_interaction(false)
            .create_or_update(db_adapter.repository())
            .await?;

        // Automatic
        assert!(PullRequestLogic::should_create_pull_request(
            &config,
            &repository,
            &creation_event
        ));
        Ok::<_, LogicError>(())
    })
    .await
}

#[actix_rt::test]
async fn test_qa_disabled_repository() -> DatabaseResult<()> {
    using_test_db("test_qa_disabled_repository", |config, pool| async move {
        let db_adapter = DatabaseAdapter::new(pool);
        let api_adapter = DummyAPIAdapter::new();
        let mut redis_adapter = DummyRedisAdapter::new();

        // Arrange
        redis_adapter
            .try_lock_resource_response
            .set_callback(Box::new(|key| {
                let inst = LockInstance::new_dummy(key);
                Ok(LockStatus::SuccessfullyLocked(inst))
            }));

        let creation_event = GhPullRequestEvent {
            action: GhPullRequestAction::Opened,
            repository: fake_gh_repo(),
            pull_request: fake_gh_pr(),
            ..Default::default()
        };

        let repo = RepositoryModel::builder_from_github(&config, &creation_event.repository)
            .default_enable_qa(false)
            .default_enable_checks(false)
            .create_or_update(db_adapter.repository())
            .await?;

        let result = handle_pull_request_opened(
            &config,
            &api_adapter,
            &db_adapter,
            &redis_adapter,
            creation_event,
        )
        .await?;
        assert_eq!(result, PullRequestOpenedStatus::Created);

        let (pr, _) = db_adapter
            .pull_request()
            .get_from_repository_and_number(repo.owner(), repo.name(), 1)
            .await?;
        let status =
            PullRequestStatus::from_database(&api_adapter, &db_adapter, &repo, &pr).await?;
        assert_eq!(status.qa_status, QaStatus::Skipped);
        assert_eq!(status.checks_status, CheckStatus::Skipped);

        Ok::<_, LogicError>(())
    })
    .await
}
