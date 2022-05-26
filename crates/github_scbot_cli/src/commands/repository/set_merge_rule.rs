use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_database2::MergeRule;
use github_scbot_types::{
    pulls::GhMergeStrategy, repository::RepositoryPath, rule_branch::RuleBranch,
};

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::ResultExt;

/// Set merge rule for a repository
#[derive(Parser)]
pub(crate) struct RepositorySetMergeRuleCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
    /// Merge strategy
    strategy: GhMergeStrategy,
}

#[async_trait(?Send)]
impl Command for RepositorySetMergeRuleCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            // Update default strategy
            pr_repo
                .set_default_strategy(owner, name, self.strategy)
                .await
                .context(DatabaseSnafu)?;

            writeln!(
                ctx.writer,
                "Default strategy updated to '{}' for repository '{}'",
                self.strategy, self.repository_path
            )
            .context(IoSnafu)?;
        } else {
            let mut mr_repo = ctx.db_adapter.merge_rules();
            mr_repo
                .delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await
                .context(DatabaseSnafu)?;
            mr_repo
                .create(
                    MergeRule::builder()
                        .repository_id(repo.id())
                        .base_branch(self.base_branch.clone())
                        .head_branch(self.head_branch.clone())
                        .strategy(self.strategy)
                        .build()
                        .unwrap(),
                )
                .await
                .context(DatabaseSnafu)?;

            writeln!(ctx.writer, "Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", self.strategy, self.repository_path, self.base_branch, self.head_branch).context(IoSnafu)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, Repository};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;
    use github_scbot_types::{pulls::GhMergeStrategy, rule_branch::RuleBranch};

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_set_merge_rule",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").default_strategy(GhMergeStrategy::Squash).build()?)
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "set-merge-rule", "owner/name", "*", "*", "merge"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Default strategy updated to 'merge' for repository 'owner/name'\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert_eq!(
                    db_adapter.repositories().get("owner", "name").await?.unwrap().default_strategy(),
                    GhMergeStrategy::Merge,
                    "repository owner/name should have a default strategy of merge"
                );

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "set-merge-rule", "owner/name", "foo", "bar", "rebase"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Merge rule created/updated with 'rebase' for repository 'owner/name' and branches 'foo' (base) <- 'bar' (head)\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert_eq!(
                    db_adapter.merge_rules().get("owner", "name", RuleBranch::Named("foo".into()), RuleBranch::Named("bar".into())).await?.unwrap().strategy(),
                    GhMergeStrategy::Rebase,
                    "repository owner/name should have a rebase strategy on foo <- bar"
                );

                Ok(())
            },
        )
        .await;
    }
}
