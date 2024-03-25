use prbot_database_interface::DatabaseError;
use prbot_models::{PullRequestRule, Repository, RuleAction, RuleBranch, RuleCondition};

use crate::testcase::db_test_case;

#[tokio::test]
async fn create() {
    db_test_case("pull_request_rule_create", |db| async move {
        assert!(matches!(
            db.pull_request_rules_create(PullRequestRule {
                repository_id: 1,
                name: "My rule".into(),
                conditions: vec![],
                actions: vec![]
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
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "My rule".into(),
                conditions: vec![],
                actions: vec![],
            })
            .await?;

        assert_eq!(rule.repository_id, repo.id);
        assert_eq!(rule.name, "My rule");
        assert_eq!(rule.conditions, vec![]);
        assert_eq!(rule.actions, vec![]);

        let rule = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "My rule #2".into(),
                conditions: vec![
                    RuleCondition::BaseBranch(RuleBranch::Named("main".into())),
                    RuleCondition::HeadBranch(RuleBranch::Named("staging".into())),
                ],
                actions: vec![RuleAction::SetAutomerge(true)],
            })
            .await?;

        assert_eq!(rule.repository_id, repo.id);
        assert_eq!(rule.name, "My rule #2");
        assert_eq!(
            rule.conditions,
            vec![
                RuleCondition::BaseBranch(RuleBranch::Named("main".into())),
                RuleCondition::HeadBranch(RuleBranch::Named("staging".into()))
            ]
        );
        assert_eq!(rule.actions, vec![RuleAction::SetAutomerge(true)]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn update() {
    db_test_case("pull_request_rule_update", |db| async move {
        assert!(matches!(
            db.pull_request_rules_update(PullRequestRule {
                repository_id: 1,
                name: "My rule".into(),
                conditions: vec![],
                actions: vec![]
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
            db.pull_request_rules_update(PullRequestRule {
                repository_id: repo.id,
                name: "My rule".into(),
                conditions: vec![],
                actions: vec![]
            },)
                .await,
            Err(DatabaseError::UnknownPullRequestRule(_))
        ));

        db.pull_request_rules_create(PullRequestRule {
            repository_id: repo.id,
            name: "My rule".into(),
            conditions: vec![
                RuleCondition::BaseBranch(RuleBranch::Named("main".into())),
                RuleCondition::HeadBranch(RuleBranch::Named("staging".into())),
            ],
            actions: vec![RuleAction::SetAutomerge(true)],
        })
        .await?;

        let rule = db
            .pull_request_rules_update(PullRequestRule {
                repository_id: repo.id,
                name: "My rule".into(),
                conditions: vec![
                    RuleCondition::BaseBranch(RuleBranch::Named("main".into())),
                    RuleCondition::HeadBranch(RuleBranch::Named("staging".into())),
                ],
                actions: vec![],
            })
            .await?;

        assert_eq!(rule.repository_id, repo.id);
        assert_eq!(rule.name, "My rule");
        assert_eq!(
            rule.conditions,
            vec![
                RuleCondition::BaseBranch(RuleBranch::Named("main".into())),
                RuleCondition::HeadBranch(RuleBranch::Named("staging".into()))
            ]
        );
        assert_eq!(rule.actions, vec![]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn get() {
    db_test_case("pull_request_rule_get", |db| async move {
        assert_eq!(db.pull_request_rules_get("me", "repo", "rule").await?, None);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let rule = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "rule".into(),
                conditions: vec![],
                actions: vec![],
            })
            .await?;

        let get_rule = db.pull_request_rules_get("me", "repo", "rule").await?;
        assert_eq!(get_rule, Some(rule));

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn delete() {
    db_test_case("pull_request_rule_delete", |db| async move {
        assert!(!db.pull_request_rules_delete("me", "repo", "rule").await?,);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        db.pull_request_rules_create(PullRequestRule {
            repository_id: repo.id,
            name: "rule".into(),
            conditions: vec![],
            actions: vec![],
        })
        .await?;

        assert!(db.pull_request_rules_delete("me", "repo", "rule").await?,);

        assert_eq!(db.pull_request_rules_get("me", "repo", "rule").await?, None);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn all() {
    db_test_case("pull_request_rules_all", |db| async move {
        assert_eq!(db.pull_request_rules_all().await?, vec![]);

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
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo1.id,
                name: "Rule 1".into(),
                actions: vec![],
                conditions: vec![],
            })
            .await?;
        let rule2 = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo1.id,
                name: "Rule 2".into(),
                ..Default::default()
            })
            .await?;
        let rule3 = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo2.id,
                name: "Rule 3".into(),
                ..Default::default()
            })
            .await?;
        let rule4 = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo2.id,
                name: "Rule 4".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            db.pull_request_rules_all().await?,
            vec![rule1, rule2, rule3, rule4]
        );

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn list() {
    db_test_case("pull_request_rule_list", |db| async move {
        assert_eq!(db.pull_request_rules_list("me", "repo").await?, vec![]);

        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        let rule1 = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 1".into(),
                ..Default::default()
            })
            .await?;
        let rule2 = db
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 2".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            db.pull_request_rules_list("me", "repo").await?,
            vec![rule1, rule2]
        );

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn cascade_repository() {
    db_test_case("pull_request_rule_cascade_repository", |db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await?;

        db.pull_request_rules_create(PullRequestRule {
            repository_id: repo.id,
            name: "Rule".into(),
            ..Default::default()
        })
        .await?;

        db.repositories_delete("me", "repo").await?;
        assert_eq!(db.pull_request_rules_all().await?, vec![]);

        Ok(())
    })
    .await;
}
