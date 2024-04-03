#![cfg(test)]

use prbot_config::Config;
use prbot_core::{
    use_cases::status::{MockSetPullRequestQaStatusInterface, SetPullRequestQaStatusInterface},
    CoreModule,
};
use prbot_database_interface::DbService;
use prbot_database_tests::{db_test_case, db_test_case_pg};
use prbot_ghapi_interface::{types::GhPullRequestEvent, ApiService, MockApiService};
use prbot_lock_interface::MockLockService;
use prbot_models::{ExternalAccount, ExternalAccountRight, PullRequest, QaStatus, Repository};
use prbot_server::server::{run_bot_server, AppContext};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

fn build_context(
    port: u16,
    core_module: CoreModule,
    api_service: Box<dyn ApiService>,
    db_service: Box<dyn DbService>,
) -> AppContext {
    let mut config = Config::from_env_no_version();
    config.server.workers_count = Some(2);
    config.server.bind_ip = "127.0.0.1".into();
    config.server.bind_port = port;

    AppContext {
        config,
        core_module,
        api_service,
        lock_service: Box::new(MockLockService::new()),
        db_service,
    }
}

fn spawn_server(
    port: u16,
    core_module: CoreModule,
    api_service: Box<dyn ApiService>,
    db_service: Box<dyn DbService>,
) {
    tokio::task::spawn_local(async move {
        let context = build_context(port, core_module, api_service, db_service);
        run_bot_server(context).await
    });
}

#[tokio::test]
#[ignore]
async fn index() {
    const PORT: u16 = 50501;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Response {
        message: String,
    }

    db_test_case("server_tests_index", |db_service| async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                let api_service = Box::new(MockApiService::new());
                spawn_server(PORT, CoreModule::builder().build(), api_service, db_service);

                let response = reqwest::get(format!("http://127.0.0.1:{PORT}"))
                    .await
                    .unwrap();
                let text: Response = response.json().await.unwrap();

                assert_eq!(
                    text,
                    Response {
                        message: "Welcome on prbot!".into()
                    }
                );
            })
            .await;

        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn metrics() {
    const PORT: u16 = 50502;

    db_test_case("server_tests_metrics", |db_service| async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                let api_service = Box::new(MockApiService::new());
                spawn_server(PORT, CoreModule::builder().build(), api_service, db_service);

                let response = reqwest::get(format!("http://127.0.0.1:{PORT}/metrics"))
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK);
            })
            .await;

        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn webhook() {
    const PORT: u16 = 50503;

    db_test_case("server_tests_webhook", |db_service| async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                let api_service = Box::new(MockApiService::new());
                spawn_server(PORT, CoreModule::builder().build(), api_service, db_service);

                let response = reqwest::Client::new()
                    .post(format!("http://127.0.0.1:{PORT}/webhook"))
                    .header("X-GitHub-Event", "pull_request")
                    .json(&GhPullRequestEvent {
                        ..Default::default()
                    })
                    .send()
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::ACCEPTED);
            })
            .await;

        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn qa_status() {
    #[derive(Serialize, Default, Debug)]
    struct QaStatusJson {
        repository_path: String,
        pull_request_numbers: Vec<u64>,
        author: String,
        status: Option<bool>,
    }

    const PORT: u16 = 50504;

    db_test_case_pg("server_tests_qa_status", |db_service| async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                // Create repository
                let repo = db_service
                    .repositories_create(Repository {
                        owner: "me".into(),
                        name: "repo".into(),
                        ..Default::default()
                    })
                    .await
                    .unwrap();

                // Create pull request
                let pr = db_service
                    .pull_requests_create(PullRequest {
                        repository_id: repo.id,
                        qa_status: QaStatus::Waiting,
                        ..Default::default()
                    })
                    .await
                    .unwrap();

                // Create user
                let exa = db_service
                    .external_accounts_create(
                        ExternalAccount {
                            username: "me".into(),
                            ..Default::default()
                        }
                        .with_generated_keys(),
                    )
                    .await
                    .unwrap();

                // Give right to user
                db_service
                    .external_account_rights_create(ExternalAccountRight {
                        repository_id: repo.id,
                        username: "me".into(),
                    })
                    .await
                    .unwrap();

                // Now generate token
                let token = exa.generate_access_token().unwrap();

                let mut set_pull_request_qa_status = MockSetPullRequestQaStatusInterface::new();
                set_pull_request_qa_status
                    .expect_run()
                    .once()
                    .withf(move |_, exa, path, numbers, author, status| {
                        exa.username == "me"
                            && path.full_name() == "me/repo"
                            && numbers == [pr.number]
                            && author == "me"
                            && *status == QaStatus::Pass
                    })
                    .return_once(|_, _, _, _, _, _| Ok(()));

                let core_module = CoreModule::builder()
                    .with_component_override::<dyn SetPullRequestQaStatusInterface>(Box::new(
                        set_pull_request_qa_status,
                    ))
                    .build();

                spawn_server(
                    PORT,
                    core_module,
                    Box::new(MockApiService::new()),
                    db_service,
                );

                let response = reqwest::Client::new()
                    .post(format!("http://127.0.0.1:{PORT}/external/set-qa-status"))
                    .bearer_auth(token)
                    .json(&QaStatusJson {
                        author: "me".into(),
                        repository_path: "me/repo".into(),
                        pull_request_numbers: vec![pr.number],
                        status: Some(true),
                    })
                    .send()
                    .await
                    .unwrap();

                assert_eq!(
                    response.status(),
                    StatusCode::ACCEPTED,
                    "{}",
                    response.text().await.unwrap()
                );
            })
            .await;

        Ok(())
    })
    .await;
}
