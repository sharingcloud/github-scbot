use github_scbot_database_interface::DatabaseError;
use github_scbot_domain_models::{MergeStrategy, Repository};

use crate::testcase::db_test_case;

#[actix_rt::test]
async fn create() {
    db_test_case("repository_create", |mut db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(repo.owner, "me");
        assert_eq!(repo.name, "repo");

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn update() {
    db_test_case("repository_update", |mut db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let repo = db
            .repositories_update(Repository {
                id: repo.id,
                owner: "me".into(),
                name: "repo2".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(repo.owner, "me");
        assert_eq!(repo.name, "repo2");

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn get() {
    db_test_case("repository_get", |mut db| async move {
        assert_eq!(db.repositories_get("me", "repo").await?, None);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let get_repo = db.repositories_get("me", "repo").await?;
        assert_eq!(get_repo, Some(repo));

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn get_from_id() {
    db_test_case("repository_get_from_id", |mut db| async move {
        assert_eq!(db.repositories_get_from_id(1).await?, None);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let get_repo = db.repositories_get_from_id(repo.id).await?;
        assert_eq!(get_repo, Some(repo));

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn delete() {
    db_test_case("repository_delete", |mut db| async move {
        assert!(!db.repositories_delete("me", "repo").await?);

        db.repositories_create(Repository {
            owner: "me".into(),
            name: "repo".into(),
            ..Default::default()
        })
        .await?;

        assert!(db.repositories_delete("me", "repo").await?);
        assert_eq!(db.repositories_get("me", "repo").await?, None);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn set_manual_interaction() {
    db_test_case("repository_set_manual_interaction", |mut db| async move {
        assert!(matches!(
            db.repositories_set_manual_interaction("me", "repo", true)
                .await,
            Err(DatabaseError::UnknownRepository(_))
        ));

        db.repositories_create(Repository {
            owner: "me".into(),
            name: "repo".into(),
            manual_interaction: false,
            ..Default::default()
        })
        .await?;

        let repo = db
            .repositories_set_manual_interaction("me", "repo", true)
            .await?;
        assert!(repo.manual_interaction);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn set_pr_title_validation_regex() {
    db_test_case(
        "repository_set_pr_title_validation_regex",
        |mut db| async move {
            assert!(matches!(
                db.repositories_set_pr_title_validation_regex("me", "repo", "[a-z]+")
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                pr_title_validation_regex: "".into(),
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_pr_title_validation_regex("me", "repo", "[a-z]+")
                .await?;
            assert_eq!(repo.pr_title_validation_regex, "[a-z]+");

            Ok(())
        },
    )
    .await;
}

#[actix_rt::test]
async fn set_default_strategy() {
    db_test_case("repository_set_default_strategy", |mut db| async move {
        assert!(matches!(
            db.repositories_set_default_strategy("me", "repo", MergeStrategy::Squash)
                .await,
            Err(DatabaseError::UnknownRepository(_))
        ));

        db.repositories_create(Repository {
            owner: "me".into(),
            name: "repo".into(),
            default_strategy: MergeStrategy::Merge,
            ..Default::default()
        })
        .await?;

        let repo = db
            .repositories_set_default_strategy("me", "repo", MergeStrategy::Squash)
            .await?;
        assert_eq!(repo.default_strategy, MergeStrategy::Squash);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn set_default_needed_reviewers_count() {
    db_test_case(
        "repository_set_default_needed_reviewers_count",
        |mut db| async move {
            assert!(matches!(
                db.repositories_set_default_needed_reviewers_count("me", "repo", 1)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                default_needed_reviewers_count: 0,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_default_needed_reviewers_count("me", "repo", 1)
                .await?;
            assert_eq!(repo.default_needed_reviewers_count, 1);

            Ok(())
        },
    )
    .await;
}

#[actix_rt::test]
async fn set_default_automerge() {
    db_test_case("repository_set_default_automerge", |mut db| async move {
        assert!(matches!(
            db.repositories_set_default_automerge("me", "repo", true)
                .await,
            Err(DatabaseError::UnknownRepository(_))
        ));

        db.repositories_create(Repository {
            owner: "me".into(),
            name: "repo".into(),
            default_automerge: false,
            ..Default::default()
        })
        .await?;

        let repo = db
            .repositories_set_default_automerge("me", "repo", true)
            .await?;
        assert!(repo.default_automerge);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn set_default_enable_qa() {
    db_test_case("repository_set_default_enable_qa", |mut db| async move {
        assert!(matches!(
            db.repositories_set_default_enable_qa("me", "repo", true)
                .await,
            Err(DatabaseError::UnknownRepository(_))
        ));

        db.repositories_create(Repository {
            owner: "me".into(),
            name: "repo".into(),
            default_enable_qa: false,
            ..Default::default()
        })
        .await?;

        let repo = db
            .repositories_set_default_enable_qa("me", "repo", true)
            .await?;
        assert!(repo.default_enable_qa);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn set_default_enable_checks() {
    db_test_case(
        "repository_set_default_enable_checks",
        |mut db| async move {
            assert!(matches!(
                db.repositories_set_default_enable_checks("me", "repo", true)
                    .await,
                Err(DatabaseError::UnknownRepository(_))
            ));

            db.repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                default_enable_checks: false,
                ..Default::default()
            })
            .await?;

            let repo = db
                .repositories_set_default_enable_checks("me", "repo", true)
                .await?;
            assert!(repo.default_enable_checks);

            Ok(())
        },
    )
    .await;
}

#[actix_rt::test]
async fn all() {
    db_test_case("repository_all", |mut db| async move {
        assert_eq!(db.repositories_all().await?, vec![]);

        let repo1 = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
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

        assert_eq!(db.repositories_all().await?, vec![repo1, repo2]);

        Ok(())
    })
    .await;
}
