//! Database tests

use super::establish_connection;
use super::models::*;

#[test]
fn create_repository() {
    let conn = establish_connection().get().unwrap();
    let repo = Repository::create(
        &conn,
        NewRepository {
            name: "TestRepo",
            owner: "me",
        },
    );

    assert_eq!(repo.id, 1);
    assert_eq!(repo.name, "TestRepo");
    assert_eq!(repo.owner, "me");
}

#[test]
fn list_repositories() {
    let conn = establish_connection().get().unwrap();
    Repository::create(
        &conn,
        NewRepository {
            name: "TestRepo",
            owner: "me",
        },
    );

    Repository::create(
        &conn,
        NewRepository {
            name: "AnotherRepo",
            owner: "me",
        },
    );

    let repos = Repository::list(&conn);
    assert_eq!(repos.len(), 2);
}
