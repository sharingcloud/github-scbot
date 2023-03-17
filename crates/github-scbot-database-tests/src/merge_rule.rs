use github_scbot_database_interface::DatabaseError;
use github_scbot_domain_models::{MergeRule, MergeStrategy, Repository, RuleBranch};

use crate::testcase::db_test_case;

#[actix_rt::test]
async fn create() {
    db_test_case("merge_rule_create", |mut db| async move {
        assert!(matches!(
            db.merge_rules_create(MergeRule {
                repository_id: 1,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge
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

        let rule = db
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;

        assert_eq!(rule.repository_id, repo.id);
        assert_eq!(rule.base_branch, RuleBranch::Wildcard);
        assert_eq!(rule.head_branch, RuleBranch::Named("hello".to_owned()));
        assert_eq!(rule.strategy, MergeStrategy::Merge);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn update() {
    db_test_case("merge_rule_update", |mut db| async move {
        assert!(matches!(
            db.merge_rules_update(MergeRule {
                repository_id: 1,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge
            },)
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
            db.merge_rules_update(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge
            },)
                .await,
            Err(DatabaseError::UnknownMergeRule(_, _))
        ));

        db.merge_rules_create(MergeRule {
            repository_id: repo.id,
            base_branch: RuleBranch::Wildcard,
            head_branch: RuleBranch::Named("hello".to_owned()),
            strategy: MergeStrategy::Merge,
        })
        .await?;

        let rule = db
            .merge_rules_update(MergeRule {
                repository_id: 1,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Squash,
            })
            .await?;

        assert_eq!(rule.repository_id, repo.id);
        assert_eq!(rule.base_branch, RuleBranch::Wildcard);
        assert_eq!(rule.head_branch, RuleBranch::Named("hello".to_owned()));
        assert_eq!(rule.strategy, MergeStrategy::Squash);
        assert_eq!(
            db.merge_rules_get(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned())
            )
            .await?,
            Some(rule)
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn get() {
    db_test_case("merge_rule_get", |mut db| async move {
        assert_eq!(
            db.merge_rules_get(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned())
            )
            .await?,
            None
        );

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let rule = db
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;

        let get_rule = db
            .merge_rules_get(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned()),
            )
            .await?;
        assert_eq!(get_rule, Some(rule));

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn delete() {
    db_test_case("merge_rule_delete", |mut db| async move {
        assert!(
            !db.merge_rules_delete(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned())
            )
            .await?,
        );

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        db.merge_rules_create(MergeRule {
            repository_id: repo.id,
            base_branch: RuleBranch::Wildcard,
            head_branch: RuleBranch::Named("hello".to_owned()),
            strategy: MergeStrategy::Merge,
        })
        .await?;

        assert!(
            db.merge_rules_delete(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned())
            )
            .await?,
        );

        assert_eq!(
            db.merge_rules_get(
                "me",
                "repo",
                RuleBranch::Wildcard,
                RuleBranch::Named("hello".to_owned())
            )
            .await?,
            None
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn all() {
    db_test_case("merge_rule_all", |mut db| async move {
        assert_eq!(db.merge_rules_all().await?, vec![]);

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

        let rule1 = db
            .merge_rules_create(MergeRule {
                repository_id: repo1.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;
        let rule2 = db
            .merge_rules_create(MergeRule {
                repository_id: repo1.id,
                base_branch: RuleBranch::Named("hi".to_owned()),
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;
        let rule3 = db
            .merge_rules_create(MergeRule {
                repository_id: repo2.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;
        let rule4 = db
            .merge_rules_create(MergeRule {
                repository_id: repo2.id,
                base_branch: RuleBranch::Named("hi".to_owned()),
                head_branch: RuleBranch::Named("hello2".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;

        assert_eq!(
            db.merge_rules_all().await?,
            vec![rule1, rule2, rule3, rule4]
        );

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn list() {
    db_test_case("merge_rule_list", |mut db| async move {
        assert_eq!(db.merge_rules_list("me", "repo").await?, vec![]);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let rule1 = db
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Wildcard,
                head_branch: RuleBranch::Named("hello".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;
        let rule2 = db
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Named("hi".to_owned()),
                head_branch: RuleBranch::Named("hello2".to_owned()),
                strategy: MergeStrategy::Merge,
            })
            .await?;

        assert_eq!(db.merge_rules_list("me", "repo").await?, vec![rule1, rule2]);

        Ok(())
    })
    .await;
}

#[actix_rt::test]
async fn cascade_repository() {
    db_test_case("merge_rule_cascade_repository", |mut db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        db.merge_rules_create(MergeRule {
            repository_id: repo.id,
            base_branch: RuleBranch::Wildcard,
            head_branch: RuleBranch::Named("hello".to_owned()),
            strategy: MergeStrategy::Merge,
        })
        .await?;

        db.repositories_delete("me", "repo").await?;
        assert_eq!(db.merge_rules_all().await?, vec![]);

        Ok(())
    })
    .await;
}
