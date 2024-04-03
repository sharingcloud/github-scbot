use async_trait::async_trait;
use prbot_models::{MergeStrategy, RepositoryPath};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait DeterminePullRequestMergeStrategyInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        base_branch: &str,
        head_branch: &str,
        default_strategy: MergeStrategy,
    ) -> Result<MergeStrategy>;
}

#[derive(Component)]
#[shaku(interface = DeterminePullRequestMergeStrategyInterface)]
pub(crate) struct DeterminePullRequestMergeStrategy;

#[async_trait]
impl DeterminePullRequestMergeStrategyInterface for DeterminePullRequestMergeStrategy {
    #[tracing::instrument(
        skip(self, ctx),
        fields(repository_path, base_branch, head_branch, default_strategy),
        ret
    )]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        base_branch: &str,
        head_branch: &str,
        default_strategy: MergeStrategy,
    ) -> Result<MergeStrategy> {
        match ctx
            .db_service
            .merge_rules_get(
                repository_path.owner(),
                repository_path.name(),
                base_branch.into(),
                head_branch.into(),
            )
            .await?
        {
            Some(r) => Ok(r.strategy),
            None => Ok(default_strategy),
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_models::{MergeRule, Repository, RuleBranch};

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn no_rule() {
        let ctx = CoreContextTest::new();
        let strategy = DeterminePullRequestMergeStrategy
            .run(
                &ctx.as_context(),
                &("me", "test").into(),
                "main",
                "abcd",
                MergeStrategy::Merge,
            )
            .await
            .unwrap();

        assert_eq!(strategy, MergeStrategy::Merge);
    }

    #[tokio::test]
    async fn custom_rule() {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.db_service
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Named("main".into()),
                head_branch: RuleBranch::Named("abcd".into()),
                strategy: MergeStrategy::Squash,
            })
            .await
            .unwrap();

        let strategy = DeterminePullRequestMergeStrategy
            .run(
                &ctx.as_context(),
                &("me", "test").into(),
                "main",
                "abcd",
                MergeStrategy::Merge,
            )
            .await
            .unwrap();

        assert_eq!(strategy, MergeStrategy::Squash);
    }
}
