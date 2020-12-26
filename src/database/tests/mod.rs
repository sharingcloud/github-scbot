//! Database tests

use crate::utils::test_init;

use super::establish_connection;
use super::models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel};

#[test]
fn create_repository() {
    test_init();

    let conn = establish_connection().unwrap().get().unwrap();
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

    let conn = establish_connection().unwrap().get().unwrap();
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

    let conn = establish_connection().unwrap().get().unwrap();
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
