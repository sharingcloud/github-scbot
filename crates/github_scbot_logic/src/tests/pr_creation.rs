use github_scbot_database::{
    get_connection, models::RepositoryModel, tests::using_test_db, Result as DatabaseResult,
};
use github_scbot_types::{
    common::{GhRepository, GhUser},
    pulls::{GhPullRequest, GhPullRequestAction, GhPullRequestEvent},
};

use super::test_config;
use crate::{pulls::should_create_pull_request, LogicError};

#[actix_rt::test]
async fn test_should_create_pull_request_manual_no_activation() -> DatabaseResult<()> {
    let config = test_config();

    using_test_db(
        &config.clone(),
        "test_db_pr_creation_no_activation",
        |pool| async move {
            let conn = get_connection(&pool)?;

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
                    ..Default::default()
                },
                ..Default::default()
            };

            let repository =
                RepositoryModel::builder_from_github(&config, &creation_event.repository)
                    .manual_interaction(true)
                    .create_or_update(&conn)?;

            // Manual interaction without activation
            assert!(!should_create_pull_request(
                &config,
                &repository,
                &creation_event
            )?);

            Ok::<_, LogicError>(())
        },
    )
    .await
}

#[actix_rt::test]
async fn test_should_create_pull_request_manual_with_activation() -> DatabaseResult<()> {
    let config = test_config();

    using_test_db(
        &config.clone(),
        "test_db_pr_creation_activation",
        |pool| async move {
            let conn = get_connection(&pool)?;

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
                    body: "Hello.\ntest-bot admin-enable".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            };

            let repository =
                RepositoryModel::builder_from_github(&config, &creation_event.repository)
                    .manual_interaction(true)
                    .create_or_update(&conn)?;

            // Manual interaction with activation
            assert!(should_create_pull_request(
                &config,
                &repository,
                &creation_event
            )?);
            Ok::<_, LogicError>(())
        },
    )
    .await
}

#[actix_rt::test]
async fn test_should_create_pull_request_automatic() -> DatabaseResult<()> {
    let config = test_config();

    using_test_db(
        &config.clone(),
        "test_db_pr_creation_automatic",
        |pool| async move {
            let conn = get_connection(&pool)?;

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
                    ..Default::default()
                },
                ..Default::default()
            };

            let repository =
                RepositoryModel::builder_from_github(&config, &creation_event.repository)
                    .manual_interaction(false)
                    .create_or_update(&conn)?;

            // Automatic
            assert!(should_create_pull_request(
                &config,
                &repository,
                &creation_event
            )?);
            Ok::<_, LogicError>(())
        },
    )
    .await
}
