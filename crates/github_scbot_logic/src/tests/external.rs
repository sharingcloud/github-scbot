//! External tests

use github_scbot_database::{
    establish_single_test_connection,
    models::{
        ExternalAccountModel, ExternalAccountRightModel, RepositoryCreation, RepositoryModel,
    },
};

use super::test_config;

#[test]
fn test_repository_right_validation() {
    let config = test_config();
    let conn = establish_single_test_connection(&config).unwrap();
    let account = ExternalAccountModel::create(&conn, "test-ext").unwrap();
    let repo = RepositoryModel::create(
        &conn,
        RepositoryCreation {
            name: "Test".to_string(),
            owner: "test".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    // No right
    assert!(ExternalAccountRightModel::get_right(&conn, &account.username, repo.id).is_err());

    // Give right
    ExternalAccountRightModel::add_right(&conn, &account.username, repo.id).unwrap();
    assert!(ExternalAccountRightModel::get_right(&conn, &account.username, repo.id).is_ok());
}
