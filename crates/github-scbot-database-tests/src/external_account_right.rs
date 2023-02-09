use github_scbot_database_interface::DatabaseError;
use github_scbot_domain_models::{ExternalAccount, ExternalAccountRight, Repository};

use crate::testcase::db_test_case;

#[actix_rt::test]
async fn create() {
    db_test_case("external_account_right_create", |mut db| async move {
        assert!(matches!(
            db.external_account_rights_create(ExternalAccountRight {
                repository_id: 1,
                username: "me".into()
            })
            .await,
            Err(DatabaseError::UnknownRepositoryId(1))
        ));

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        assert!(matches!(
            db.external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "me".into()
            })
            .await,
            Err(DatabaseError::UnknownExternalAccount(_))
        ));

        let exa = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let exr = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: exa.username.clone(),
            })
            .await?;

        assert_eq!(exr.username, exa.username);
        assert_eq!(exr.repository_id, repo.id);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn get() {
    db_test_case("external_account_right_get", |mut db| async move {
        assert_eq!(
            db.external_account_rights_get("me", "repo", "me").await?,
            None
        );

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;
        let exa = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let exr = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: exa.username,
            })
            .await?;

        let get_exr = db.external_account_rights_get("me", "repo", "me").await?;
        assert_eq!(Some(exr), get_exr);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn delete() {
    db_test_case("external_account_right_delete", |mut db| async move {
        assert!(
            !db.external_account_rights_delete("me", "repo", "me")
                .await?
        );

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;
        let exa = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo.id,
            username: exa.username,
        })
        .await?;

        assert!(
            db.external_account_rights_delete("me", "repo", "me")
                .await?
        );
        assert_eq!(
            db.external_account_rights_get("me", "repo", "me").await?,
            None
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn delete_all() {
    db_test_case("external_account_right_delete_all", |mut db| async move {
        assert!(!db.external_account_rights_delete_all("me").await?);

        let repo1 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo1".into(),
                ..Default::default()
            })
            .await?;
        let repo2 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo2".into(),
                ..Default::default()
            })
            .await?;
        let exa1 = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
        let exa2 = db
            .external_accounts_create(ExternalAccount {
                username: "me2".into(),
                ..Default::default()
            })
            .await?;

        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo1.id,
            username: exa1.username.clone(),
        })
        .await?;
        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo2.id,
            username: exa1.username.clone(),
        })
        .await?;
        let exr3 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo2.id,
                username: exa2.username.clone(),
            })
            .await?;

        assert!(db.external_account_rights_delete_all("me").await?);
        assert_eq!(db.external_account_rights_list("me").await?, vec![]);
        assert_eq!(db.external_account_rights_list("me2").await?, vec![exr3]);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn list() {
    db_test_case("external_account_right_list", |mut db| async move {
        assert_eq!(db.external_account_rights_list("me").await?, vec![]);

        let repo1 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo1".into(),
                ..Default::default()
            })
            .await?;
        let repo2 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo2".into(),
                ..Default::default()
            })
            .await?;
        let exa1 = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let exr1 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo1.id,
                username: exa1.username.clone(),
            })
            .await?;
        let exr2 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo2.id,
                username: exa1.username.clone(),
            })
            .await?;

        assert_eq!(
            db.external_account_rights_list("me").await?,
            vec![exr1, exr2]
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn all() {
    db_test_case("external_account_right_all", |mut db| async move {
        assert_eq!(db.external_account_rights_all().await?, vec![]);

        let repo1 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo1".into(),
                ..Default::default()
            })
            .await?;
        let repo2 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo2".into(),
                ..Default::default()
            })
            .await?;
        let exa1 = db
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;
        let exa2 = db
            .external_accounts_create(ExternalAccount {
                username: "her".into(),
                ..Default::default()
            })
            .await?;

        let exr1 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo1.id,
                username: exa1.username.clone(),
            })
            .await?;
        let exr2 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo2.id,
                username: exa1.username.clone(),
            })
            .await?;
        let exr3 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo1.id,
                username: exa2.username.clone(),
            })
            .await?;
        let exr4 = db
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo2.id,
                username: exa2.username.clone(),
            })
            .await?;

        assert_eq!(
            db.external_account_rights_all().await?,
            vec![exr3, exr4, exr1, exr2]
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn cascade_external_account() {
    db_test_case(
        "external_account_right_cascade_external_account",
        |mut db| async move {
            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;
            let exa = db
                .external_accounts_create(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await?;
            db.external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: exa.username,
            })
            .await?;

            // On account deletion, rights should be dropped
            db.external_accounts_delete("me").await?;
            assert_eq!(db.external_account_rights_all().await?, vec![]);

            Ok(())
        },
    )
    .await
}

#[actix_rt::test]
async fn cascade_repository() {
    db_test_case(
        "external_account_right_cascade_repository",
        |mut db| async move {
            let repo = db
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "repo".into(),
                    ..Default::default()
                })
                .await?;
            let exa = db
                .external_accounts_create(ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                })
                .await?;
            db.external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: exa.username,
            })
            .await?;

            // On repository deletion, rights should be dropped
            db.repositories_delete("me", "repo").await?;
            assert_eq!(db.external_account_rights_list("me").await?, vec![]);

            Ok(())
        },
    )
    .await
}
