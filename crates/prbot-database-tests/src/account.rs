use prbot_database_interface::DatabaseError;
use prbot_models::Account;

use crate::testcase::db_test_case;

#[tokio::test]
async fn default_account_is_not_admin() {
    db_test_case("account_default_account_is_not_admin", |db| async move {
        let account = db
            .accounts_create(Account {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
        assert_eq!(account.username, "me");
        assert!(!account.is_admin);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn create() {
    db_test_case("account_create", |db| async move {
        let account = db
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;
        assert!(account.is_admin);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn update() {
    db_test_case("account_update", |db| async move {
        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        let account = db
            .accounts_update(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;
        assert!(account.is_admin);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn get() {
    db_test_case("account_get", |db| async move {
        assert_eq!(db.accounts_get("me").await?, None);

        let account = db
            .accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;

        let get_account = db.accounts_get("me").await?;
        assert_eq!(Some(account), get_account);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn delete() {
    db_test_case("account_delete", |db| async move {
        assert!(!db.accounts_delete("me").await?);

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        assert!(db.accounts_delete("me").await?);
        assert_eq!(db.accounts_get("me").await?, None);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn list_admins() {
    db_test_case("account_list_admins", |db| async move {
        assert_eq!(db.accounts_list_admins().await?, vec![]);

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;
        let account2 = db
            .accounts_create(Account {
                username: "him".into(),
                is_admin: true,
            })
            .await?;
        let account3 = db
            .accounts_create(Account {
                username: "her".into(),
                is_admin: true,
            })
            .await?;

        let admins = db.accounts_list_admins().await?;
        assert_eq!(admins, vec![account3, account2]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn all() {
    db_test_case("account_all", |db| async move {
        assert_eq!(db.accounts_all().await?, vec![]);

        let account1 = db
            .accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;
        let account2 = db
            .accounts_create(Account {
                username: "him".into(),
                is_admin: false,
            })
            .await?;
        let account3 = db
            .accounts_create(Account {
                username: "her".into(),
                is_admin: false,
            })
            .await?;

        let accounts = db.accounts_all().await?;
        assert_eq!(accounts, vec![account3, account2, account1]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn set_is_admin() {
    db_test_case("account_set_is_admin", |db| async move {
        assert!(matches!(
            db.accounts_set_is_admin("me", true).await,
            Err(DatabaseError::UnknownAccount(_))
        ));

        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await?;

        let account1 = db.accounts_set_is_admin("me", true).await?;
        assert!(account1.is_admin);

        let account1 = db.accounts_set_is_admin("me", false).await?;
        assert!(!account1.is_admin);

        Ok(())
    })
    .await;
}
