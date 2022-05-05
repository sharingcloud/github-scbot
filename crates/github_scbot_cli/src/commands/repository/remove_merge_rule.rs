use std::io::Write;

use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_sentry::eyre::{eyre::eyre, Result};
use github_scbot_types::{repository::RepositoryPath, rule_branch::RuleBranch};

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// remove merge rule for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "remove-merge-rule")]
pub(crate) struct RepositoryRemoveMergeRuleCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
    /// base branch name.
    #[argh(positional)]
    base_branch: RuleBranch,
    /// head branch name.
    #[argh(positional)]
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
            return Err(eyre!("Cannot remove default strategy"));
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
                .await?;
            if found {
                writeln!(
                    ctx.writer,
                    "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            } else {
                writeln!(
                    ctx.writer,
                    "Unknown merge rule for repository '{}' and branches '{}' (base) <- '{}' (head).",
                    self.repository_path, self.base_branch, self.head_branch
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{
        use_temporary_db, DbService, DbServiceImplPool, MergeRule, Repository,
    };
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;
    use github_scbot_types::rule_branch::RuleBranch;

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
