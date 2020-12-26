//! Database tests

use crate::utils::test_init;

use super::establish_single_connection;
use super::models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel};

#[test]
fn create_repository() {
    test_init();

    let conn = establish_single_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    assert_eq!(repo.id, 1);
    assert_eq!(repo.name, "TestRepo");
    assert_eq!(repo.owner, "me");
}

#[test]
fn list_repositories() {
    test_init();

    let conn = establish_single_connection().unwrap();
    RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "AnotherRepo",
            owner: "me",
        },
    )
    .unwrap();

    let repos = RepositoryModel::list(&conn).unwrap();
    assert_eq!(repos.len(), 2);
}

#[test]
fn create_pull_request() {
    test_init();

    let conn = establish_single_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    let pr = PullRequestModel::create(
        &conn,
        &PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto",
            automerge: false,
            check_status: None,
            step: None,
        },
    )
    .unwrap();

    assert_eq!(pr.id, 1);
    assert_eq!(pr.repository_id, repo.id);
    assert_eq!(pr.number, 1234);
}

#[test]
fn test_export_models_to_json() {
    use super::import_export::export_models_to_json;

    test_init();

    let conn = establish_single_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    PullRequestModel::create(
        &conn,
        &PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto",
            ..PullRequestCreation::default()
        },
    )
    .unwrap();

    let mut buffer = Vec::new();
    export_models_to_json(&conn, &mut buffer).unwrap();

    let buffer_string = String::from_utf8(buffer).unwrap();
    assert!(buffer_string.contains(r#""name": "TestRepo""#));
    assert!(buffer_string.contains(r#""number": 1234"#));
}

#[test]
fn test_import_models_to_json() {
    use super::import_export::import_models_from_json;

    test_init();

    let conn = establish_single_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        &RepositoryCreation {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    PullRequestModel::create(
        &conn,
        &PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto",
            ..PullRequestCreation::default()
        },
    )
    .unwrap();

    let sample = r#"
        {
            "repositories": [
                {
                    "id": 1,
                    "name": "TestRepo",
                    "owner": "me",
                    "pr_title_validation_regex": "[a-z]*"
                },
                {
                    "id": 2,
                    "name": "AnotherRepo",
                    "owner": "me",
                    "pr_title_validation_regex": ""
                }
            ],
            "pull_requests": [
                {
                    "id": 1,
                    "repository_id": 1,
                    "number": 1234,
                    "name": "Tutu",
                    "automerge": false,
                    "step": "step/awaiting-review",
                    "check_status": null,
                    "status_comment_id": 1,
                    "qa_status": null,
                    "wip": false,
                    "required_reviewers": ""
                },
                {
                    "id": 2,
                    "repository_id": 1,
                    "number": 1235,
                    "name": "Tata",
                    "automerge": true,
                    "step": "step/wip",
                    "check_status": "pass",
                    "status_comment_id": 0,
                    "qa_status": "pass",
                    "wip": true,
                    "required_reviewers": ""
                }
            ]
        }
    "#;

    import_models_from_json(&conn, sample.as_bytes()).unwrap();

    let rep_1 = RepositoryModel::get_from_name(&conn, "TestRepo", "me").unwrap();
    let rep_2 = RepositoryModel::get_from_name(&conn, "AnotherRepo", "me").unwrap();
    let pr_1 = PullRequestModel::get_from_number(&conn, rep_1.id, 1234).unwrap();
    let pr_2 = PullRequestModel::get_from_number(&conn, rep_1.id, 1235).unwrap();

    assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
    assert_eq!(rep_2.pr_title_validation_regex, "");
    assert_eq!(pr_1.name, "Tutu");
    assert_eq!(pr_1.automerge, false);
    assert_eq!(pr_2.name, "Tata");
    assert_eq!(pr_2.automerge, true);
}
