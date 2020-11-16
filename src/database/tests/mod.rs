//! Database tests

use super::establish_connection;
use super::models::*;

#[test]
fn create_repository() {
    let conn = establish_connection().unwrap().get().unwrap();
    let repo = Repository::create(
        &conn,
        NewRepository {
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
    let conn = establish_connection().unwrap().get().unwrap();
    Repository::create(
        &conn,
        NewRepository {
            name: "TestRepo",
            owner: "me",
        },
    )
    .unwrap();

    Repository::create(
        &conn,
        NewRepository {
            name: "AnotherRepo",
            owner: "me",
        },
    )
    .unwrap();

    let repos = Repository::list(&conn).unwrap();
    assert_eq!(repos.len(), 2);
}
