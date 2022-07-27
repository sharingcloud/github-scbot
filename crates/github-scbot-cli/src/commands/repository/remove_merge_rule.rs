use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::{repository::RepositoryPath, rule_branch::RuleBranch};

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::{whatever, ResultExt};

/// Remove merge rule for a repository
#[derive(Parser)]
pub(crate) struct RepositoryRemoveMergeRuleCommand {
    /// Repository path (e.g. `MyOrganization/my-project`)
    repository_path: RepositoryPath,
    /// Base branch name
    base_branch: RuleBranch,
    /// Head branch name
    head_branch: RuleBranch,
}

#[async_trait(?Send)]
impl Command for RepositoryRemoveMergeRuleCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let _repo =
            CliDbExt::get_existing_repository(&mut *ctx.db_adapter.repositories(), owner, name)
                .await?;

        if self.base_branch == RuleBranch::Wildcard && self.head_branch == RuleBranch::Wildcard {
            whatever!("Cannot remove default strategy");
        } else {
            let found = ctx
                .db_adapter
                .merge_rules()
                .delete(
                    owner,
                    name,
                    self.base_branch.clone(),
                    self.head_branch.clone(),
                )
                .await
                .context(DatabaseSnafu)?;
            if found {
                writeln!(
                    ctx.writer,
                    "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                    self.repository_path, self.base_branch, self.head_branch
                )
                .context(IoSnafu)?;
            } else {
                writeln!(
                    ctx.writer,
                    "Unknown merge rule for repository '{}' and branches '{}' (base) <- '{}' (head).",
                    self.repository_path, self.base_branch, self.head_branch
                )
                .context(IoSnafu)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_core::types::rule_branch::RuleBranch;
    use github_scbot_database2::{
        use_temporary_db, DbService, DbServiceImplPool, MergeRule, Repository,
    };
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_remove_merge_rule",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                let repo = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "remove-merge-rule", "owner/name", "foo", "bar"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Unknown merge rule for repository 'owner/name' and branches 'foo' (base) <- 'bar' (head).\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .merge_rules()
                    .create(
                        MergeRule::builder()
                            .with_repository(&repo)
                            .base_branch(RuleBranch::Named("foo".into()))
                            .head_branch(RuleBranch::Named("bar".into()))
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "remove-merge-rule", "owner/name", "foo", "bar"],
                )
                .await?;

                assert_eq!(
                    output,
                    "Merge rule for repository 'owner/name' and branches 'foo' (base) <- 'bar' (head) deleted.\n"
                );

                let db_adapter = DbServiceImplPool::new(pool.clone());
                assert!(
                    db_adapter.merge_rules().get("owner", "name", RuleBranch::Named("foo".into()), RuleBranch::Named("bar".into())).await?.is_none(),
                    "merge rule should have been removed"
                );

                Ok(())
            },
        )
        .await;
    }
}
