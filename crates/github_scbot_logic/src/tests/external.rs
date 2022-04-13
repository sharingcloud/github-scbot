//! External tests

use github_scbot_database2::DatabaseError;

use crate::LogicError;

#[actix_rt::test]
async fn test_repository_right_validation() -> Result<(), DatabaseError> {
    // using_test_db("test_logic_external", |config, pool| async move {
    //     let db_adapter = DatabaseAdapter::new(pool);
    //     let account = ExternalAccountModel::builder("test-ext")
    //         .generate_keys()
    //         .create_or_update(db_adapter.external_account())
    //         .await?;
    //     let repo = RepositoryModel::builder(&config, "test", "Test")
    //         .create_or_update(db_adapter.repository())
    //         .await?;

    //     // No right
    //     assert!(db_adapter
    //         .external_account_right()
    //         .get_right(&account.username, &repo)
    //         .await
    //         .is_err());

    //     // Give right
    //     db_adapter
    //         .external_account_right()
    //         .add_right(&account.username, &repo)
    //         .await?;
    //     assert!(db_adapter
    //         .external_account_right()
    //         .get_right(&account.username, &repo)
    //         .await
    //         .is_ok());

    //     Ok::<_, LogicError>(())
    // })
    // .await
    todo!()
}
