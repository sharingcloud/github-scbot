//! External tests

use github_scbot_database::{
    establish_single_test_connection,
    models::{ExternalAccountModel, ExternalAccountRightModel, RepositoryModel},
};

use super::test_config;

#[test]
fn test_repository_right_validation() {
    let config = test_config();
    let conn = establish_single_test_connection(&config).unwrap();
    let account = ExternalAccountModel::builder("test-ext")
        .generate_keys()
        .create_or_update(&conn)
        .unwrap();
    let repo = RepositoryModel::builder(&config, "test", "Test")
        .create_or_update(&conn)
        .unwrap();

    // No right
    assert!(ExternalAccountRightModel::get_right(&conn, &account.username, &repo).is_err());

    // Give right
    ExternalAccountRightModel::add_right(&conn, &account.username, &repo).unwrap();
    assert!(ExternalAccountRightModel::get_right(&conn, &account.username, &repo).is_ok());
}
