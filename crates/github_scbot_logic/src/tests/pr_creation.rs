use github_scbot_conf::Config;
use github_scbot_ghapi::adapter::MockApiService;
use github_scbot_redis::{LockInstance, LockStatus, MockRedisService};
use github_scbot_types::{
    checks::{GhCheckConclusion, GhCheckStatus, GhCheckSuite},
    common::{GhApplication, GhRepository, GhUser},
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent, GhPullRequestShort},
};

use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, Repository, Result};

use crate::pulls::{handle_pull_request_opened, PullRequestLogic, PullRequestOpenedStatus};

#[actix_rt::test]
async fn test_should_create_pull_request_manual_no_activation() -> Result<()> {
    let config = Config::from_env();
    use_temporary_db(
        config,
        "test_should_create_pull_request_manual_no_activation",
        |config, pool| async move {
            let db_adapter = DbServiceImplPool::new(pool);
            let repository = Repository::builder()
                .with_config(&config)
                .name("name")
                .owner("owner")
                .manual_interaction(true)
                .build()
                .unwrap();
            let repository = db_adapter.repositories().create(repository).await?;

            let event = GhPullRequestEvent {
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

            assert!(
                !PullRequestLogic::should_create_pull_request(&config, &repository, &event),
                "it should not be possible to create a pull request on PR event"
            );

            Ok(())
        },
    )
    .await;

    Ok(())
}

#[actix_rt::test]
async fn test_should_create_pull_request_manual_with_activation() -> Result<()> {
    let config = Config::from_env();
    use_temporary_db(
        config,
        "test_should_create_pull_request_manual_with_activation",
        |config, pool| async move {
            let db_adapter = DbServiceImplPool::new(pool);
            let repository = Repository::builder()
                .with_config(&config)
                .name("name")
                .owner("owner")
                .manual_interaction(true)
                .build()
                .unwrap();
            let repository = db_adapter.repositories().create(repository).await?;

            let event = GhPullRequestEvent {
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
                    body: Some("Hello.\nbot admin-enable".to_string()),
                    ..GhPullRequest::default()
                },
                ..GhPullRequestEvent::default()
            };

            assert!(
                PullRequestLogic::should_create_pull_request(&config, &repository, &event),
                "it should be possible to create a pull request on PR event"
            );

            Ok(())
        },
    )
    .await;

    Ok(())
}

#[actix_rt::test]
async fn test_should_create_pull_request_automatic() -> Result<()> {
    let config = Config::from_env();
    use_temporary_db(
        config,
        "test_should_create_pull_request_automatic",
        |config, pool| async move {
            let db_adapter = DbServiceImplPool::new(pool);
            let repository = Repository::builder()
                .with_config(&config)
                .name("name")
                .owner("owner")
                .manual_interaction(false)
                .build()
                .unwrap();
            let repository = db_adapter.repositories().create(repository).await?;

            let event = GhPullRequestEvent {
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

            assert!(
                PullRequestLogic::should_create_pull_request(&config, &repository, &event),
                "it should be possible to create a pull request on PR event"
            );

            Ok(())
        },
    )
    .await;

    Ok(())
}

#[actix_rt::test]
async fn test_qa_disabled_repository() -> Result<()> {
    let config = Config::from_env();
    use_temporary_db(
        config,
        "test_qa_disabled_repository",
        |config, pool| async move {
            let db_adapter = DbServiceImplPool::new(pool);
            let repository = Repository::builder()
                .with_config(&config)
                .name("name")
                .owner("owner")
                .manual_interaction(false)
                .default_enable_qa(false)
                .default_enable_checks(true)
                .build()
                .unwrap();
            db_adapter.repositories().create(repository).await?;

            let event = GhPullRequestEvent {
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

            let mut api_adapter = MockApiService::new();
            let mut redis_adapter = MockRedisService::new();
            redis_adapter
                .expect_wait_lock_resource()
                .times(1)
                .returning(|_, _| {
                    Ok(LockStatus::SuccessfullyLocked(LockInstance::new_dummy(
                        "pouet",
                    )))
                });

            api_adapter
                .expect_pulls_get()
                .times(1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        mergeable: Some(true),
                        ..Default::default()
                    })
                });

            api_adapter
                .expect_pull_reviews_list()
                .times(1)
                .return_once(|_, _, _| Ok(vec![]));

            api_adapter
                .expect_check_suites_list()
                .times(1)
                .return_once(|_, _, _| {
                    Ok(vec![GhCheckSuite {
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        pull_requests: vec![GhPullRequestShort {
                            number: 1,
                            ..Default::default()
                        }],
                        app: GhApplication {
                            slug: "github-actions".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }])
                });

            api_adapter
                .expect_issue_labels_list()
                .times(1)
                .return_once(|_, _, _| Ok(vec![]));

            api_adapter
                .expect_issue_labels_replace_all()
                .withf(|owner, name, pr_number, labels| {
                    owner == "owner"
                        && name == "name"
                        && *pr_number == 1u64
                        && labels == ["step/awaiting-review"]
                })
                .times(1)
                .return_once(|_, _, _, _| Ok(()));

            api_adapter
                .expect_comments_post()
                .times(1)
                .return_once(|_, _, _, _| Ok(1));

            api_adapter
                .expect_commit_statuses_update()
                .times(1)
                .return_once(|_, _, _, _, _, _| Ok(()));

            let result = handle_pull_request_opened(
                &config,
                &api_adapter,
                &db_adapter,
                &redis_adapter,
                event,
            )
            .await?;
            assert_eq!(result, PullRequestOpenedStatus::Created);

            Ok(())
        },
    )
    .await;

    Ok(())
}
