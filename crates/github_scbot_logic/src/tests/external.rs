//! External tests

use github_scbot_database::{
    models::{ExternalAccountModel, ExternalAccountRightModel, RepositoryModel},
    tests::using_test_db,
    Result,
};

use super::test_config;
use crate::LogicError;

#[actix_rt::test]
async fn test_repository_right_validation() -> Result<()> {
    let config = test_config();
    using_test_db(&config.clone(), "test_logic_external", |pool| async move {
        let conn = pool.get().unwrap();
        let account = ExternalAccountModel::builder("test-ext")
            .generate_keys()
            .create_or_update(&conn)?;
        let repo = RepositoryModel::builder(&config, "test", "Test").create_or_update(&conn)?;

        // No right
        assert!(ExternalAccountRightModel::get_right(&conn, &account.username, &repo).is_err());

        // Give right
        ExternalAccountRightModel::add_right(&conn, &account.username, &repo)?;
        assert!(ExternalAccountRightModel::get_right(&conn, &account.username, &repo).is_ok());

        Ok::<_, LogicError>(())
    })
    .await
}
