use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch};
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{
    fields::{GhMergeStrategyDecode, RuleBranchDecode},
    Repository,
};

#[derive(
    SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize, PartialEq, Eq,
)]
#[builder(default, setter(into))]
pub struct MergeRule {
    #[get]
    pub(crate) repository_id: u64,
    #[get_ref]
    pub(crate) base_branch: RuleBranch,
    #[get_ref]
    pub(crate) head_branch: RuleBranch,
    #[get]
    pub(crate) strategy: GhMergeStrategy,
}

impl MergeRule {
    pub fn builder() -> MergeRuleBuilder {
        MergeRuleBuilder::default()
    }

    pub fn set_repository_id(&mut self, id: u64) {
        self.repository_id = id;
    }
}

impl MergeRuleBuilder {
    pub fn with_repository(&mut self, repository: &Repository) -> &mut Self {
        self.repository_id = Some(repository.id());
        self
    }
}

impl<'r> FromRow<'r, PgRow> for MergeRule {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            base_branch: row.try_get::<RuleBranchDecode, _>("base_branch")?.clone(),
            head_branch: row.try_get::<RuleBranchDecode, _>("head_branch")?.clone(),
            strategy: *row.try_get::<GhMergeStrategyDecode, _>("strategy")?,
        })
    }
}

#[cfg(test)]
mod new_tests {
    use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch};

    use crate::{utils::db_test_case, MergeRule, Repository};

    #[actix_rt::test]
    async fn create() {
        db_test_case("merge_rule_create", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            let rule = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
                .await?;

            assert_eq!(rule.repository_id, repo.id());
            assert_eq!(rule.base_branch, RuleBranch::Wildcard);
            assert_eq!(rule.head_branch, RuleBranch::Named("hello".to_owned()));
            assert_eq!(rule.strategy, GhMergeStrategy::Merge);

            Ok(())
        })
        .await;
    }

    #[actix_rt::test]
    async fn update() {
        db_test_case("merge_rule_update", |mut db| async move {
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            db.merge_rules_create(
                MergeRule::builder()
                    .repository_id(repo.id())
                    .base_branch(RuleBranch::Wildcard)
                    .head_branch(RuleBranch::Named("hello".to_owned()))
                    .strategy(GhMergeStrategy::Merge)
                    .build()?,
            )
            .await?;

            let rule = db
                .merge_rules_update(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Squash)
                        .build()?,
                )
                .await?;

            assert_eq!(rule.repository_id, repo.id());
            assert_eq!(rule.base_branch, RuleBranch::Wildcard);
            assert_eq!(rule.head_branch, RuleBranch::Named("hello".to_owned()));
            assert_eq!(rule.strategy, GhMergeStrategy::Squash);
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
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

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

            let rule = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
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
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

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

            db.merge_rules_create(
                MergeRule::builder()
                    .repository_id(repo.id())
                    .base_branch(RuleBranch::Wildcard)
                    .head_branch(RuleBranch::Named("hello".to_owned()))
                    .strategy(GhMergeStrategy::Merge)
                    .build()?,
            )
            .await?;

            assert_eq!(
                db.merge_rules_delete(
                    "me",
                    "repo",
                    RuleBranch::Wildcard,
                    RuleBranch::Named("hello".to_owned())
                )
                .await?,
                true
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
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;
            let repo2 = db
                .repositories_create(Repository::builder().owner("me").name("repo2").build()?)
                .await?;

            let rule1 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo1.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
                .await?;
            let rule2 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo1.id())
                        .base_branch(RuleBranch::Named("hi".to_owned()))
                        .head_branch(RuleBranch::Named("hello2".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
                .await?;
            let rule3 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo2.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
                .await?;
            let rule4 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo2.id())
                        .base_branch(RuleBranch::Named("hi".to_owned()))
                        .head_branch(RuleBranch::Named("hello2".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
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
            let repo = db
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            assert_eq!(db.merge_rules_list("me", "repo").await?, vec![]);

            let rule1 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(RuleBranch::Wildcard)
                        .head_branch(RuleBranch::Named("hello".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
                .await?;
            let rule2 = db
                .merge_rules_create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(RuleBranch::Named("hi".to_owned()))
                        .head_branch(RuleBranch::Named("hello2".to_owned()))
                        .strategy(GhMergeStrategy::Merge)
                        .build()?,
                )
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
                .repositories_create(Repository::builder().owner("me").name("repo").build()?)
                .await?;

            db.merge_rules_create(
                MergeRule::builder()
                    .repository_id(repo.id())
                    .base_branch(RuleBranch::Wildcard)
                    .head_branch(RuleBranch::Named("hello".to_owned()))
                    .strategy(GhMergeStrategy::Merge)
                    .build()?,
            )
            .await?;

            db.repositories_delete("me", "repo").await?;
            assert_eq!(db.merge_rules_all().await?, vec![]);

            Ok(())
        })
        .await;
    }
}
