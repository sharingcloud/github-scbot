use prbot_database_interface::DatabaseError;
use prbot_models::ExternalAccount;

use crate::testcase::db_test_case;

#[tokio::test]
async fn create_no_keys() {
    db_test_case("external_account_create_no_keys", |db| async move {
        let exa = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
        assert_eq!(exa.username, "me");
        assert_eq!(exa.public_key, "");
        assert_eq!(exa.private_key, "");

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn create_keys() {
    db_test_case("external_account_create_keys", |db| async move {
        let exa = db
            .external_accounts_create(
                ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;
        assert_eq!(exa.username, "me");
        assert_ne!(exa.public_key, "");
        assert_ne!(exa.private_key, "");

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn update() {
    db_test_case("external_account_update", |db| async move {
        assert!(matches!(
            db.external_accounts_update(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await,
            Err(DatabaseError::UnknownExternalAccount(_))
        ));

        db.external_accounts_create(ExternalAccount {
            username: "me".into(),
            ..Default::default()
        })
        .await?;
        let exa = db
            .external_accounts_update(
                ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;
        assert_eq!(exa.username, "me");
        assert_ne!(exa.public_key, "");
        assert_ne!(exa.private_key, "");

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn set_keys() {
    db_test_case("external_account_set_keys", |db| async move {
        assert!(matches!(
            db.external_accounts_set_keys("me", "one", "two").await,
            Err(DatabaseError::UnknownExternalAccount(_))
        ));

        db.external_accounts_create(ExternalAccount {
            username: "me".into(),
            ..Default::default()
        })
        .await?;

        let exa = db.external_accounts_set_keys("me", "one", "two").await?;
        assert_eq!(exa.username, "me");
        assert_eq!(exa.public_key, "one");
        assert_eq!(exa.private_key, "two");

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn get() {
    db_test_case("external_account_get", |db| async move {
        assert_eq!(db.external_accounts_get("me").await?, None);

        let exa = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let get_exa = db.external_accounts_get("me").await?;
        assert_eq!(Some(exa), get_exa);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn delete() {
    db_test_case("external_account_delete", |db| async move {
        assert!(!db.external_accounts_delete("me").await?);

        db.external_accounts_create(ExternalAccount {
            username: "me".into(),
            ..Default::default()
        })
        .await?;

        assert!(db.external_accounts_delete("me").await?);

        let get_exa = db.external_accounts_get("me").await?;
        assert_eq!(get_exa, None);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn all() {
    db_test_case("external_account_all", |db| async move {
        assert_eq!(db.external_accounts_all().await?, vec![]);

        let exa1 = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
        let exa2 = db
            .external_accounts_create(ExternalAccount {
                username: "him".into(),
                ..Default::default()
            })
            .await?;
        let exa3 = db
            .external_accounts_create(ExternalAccount {
                username: "her".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(db.external_accounts_all().await?, vec![exa3, exa2, exa1]);

        Ok(())
    })
    .await;
}
