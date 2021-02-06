use github_scbot_types::reviews::GHReviewState;

use super::import_export::{export_models_to_json, import_models_from_json};
use crate::{
    establish_single_test_connection,
    models::{
        PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel, ReviewCreation,
        ReviewModel,
    },
};

fn test_init() {}

#[test]
fn create_repository() {
    test_init();

    let conn = establish_single_test_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..Default::default()
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

    let conn = establish_single_test_connection().unwrap();
    RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..Default::default()
        },
    )
    .unwrap();

    RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "AnotherRepo".into(),
            owner: "me".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let repos = RepositoryModel::list(&conn).unwrap();
    assert_eq!(repos.len(), 2);
}

#[test]
fn create_pull_request() {
    test_init();

    let conn = establish_single_test_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".to_string(),
            owner: "me".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(pr.id, 1);
    assert_eq!(pr.repository_id, repo.id);
    assert_eq!(pr.get_number(), 1234);
}

#[test]
fn test_export_models_to_json() {
    test_init();

    let conn = establish_single_test_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let pr = PullRequestModel::create(
        &conn,
        PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto".into(),
            ..Default::default()
        },
    )
    .unwrap();

    ReviewModel::create(
        &conn,
        ReviewCreation {
            pull_request_id: pr.id,
            required: true,
            state: GHReviewState::Commented.to_string(),
            username: "toto",
        },
    )
    .unwrap();

    let mut buffer = Vec::new();
    export_models_to_json(&conn, &mut buffer).unwrap();

    let buffer_string = String::from_utf8(buffer).unwrap();
    assert!(buffer_string.contains(r#""name": "TestRepo""#));
    assert!(buffer_string.contains(r#""number": 1234"#));
    assert!(buffer_string.contains(r#""username": "toto"#));
}

#[test]
fn test_import_models_from_json() {
    test_init();

    let conn = establish_single_test_connection().unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "TestRepo".into(),
            owner: "me".into(),
            ..Default::default()
        },
    )
    .unwrap();

    PullRequestModel::create(
        &conn,
        PullRequestCreation {
            repository_id: repo.id,
            number: 1234,
            name: "Toto".into(),
            ..Default::default()
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
                    "pr_title_validation_regex": "[a-z]*",
                    "default_needed_reviewers_count": 2
                },
                {
                    "id": 2,
                    "name": "AnotherRepo",
                    "owner": "me",
                    "pr_title_validation_regex": "",
                    "default_needed_reviewers_count": 3
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
                    "needed_reviewers_count": 2,
                    "locked": false,
                    "merged": false
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
                    "needed_reviewers_count": 2,
                    "locked": true,
                    "merged": false
                }
            ],
            "reviews": [
                {
                    "id": 1,
                    "pull_request_id": 1,
                    "username": "tutu",
                    "state": "commented",
                    "required": true
                }
            ]
        }
    "#;

    import_models_from_json(&conn, sample.as_bytes()).unwrap();

    let rep_1 = RepositoryModel::get_from_owner_and_name(&conn, "me", "TestRepo").unwrap();
    let rep_2 = RepositoryModel::get_from_owner_and_name(&conn, "me", "AnotherRepo").unwrap();
    let pr_1 = PullRequestModel::get_from_repository_id_and_number(&conn, rep_1.id, 1234).unwrap();
    let pr_2 = PullRequestModel::get_from_repository_id_and_number(&conn, rep_1.id, 1235).unwrap();
    let review_1 = ReviewModel::get_from_pull_request_and_username(&conn, pr_1.id, "tutu").unwrap();

    assert_eq!(rep_1.pr_title_validation_regex, "[a-z]*");
    assert_eq!(rep_2.pr_title_validation_regex, "");
    assert_eq!(pr_1.name, "Tutu");
    assert_eq!(pr_1.automerge, false);
    assert_eq!(pr_2.name, "Tata");
    assert_eq!(pr_2.automerge, true);
    assert_eq!(review_1.required, true);
    assert_eq!(review_1.get_review_state(), GHReviewState::Commented);
}
