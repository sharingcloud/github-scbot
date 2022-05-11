use std::io::Write;

use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::repository::RepositoryPath;

use crate::errors::{DatabaseSnafu, IoSnafu};
use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};
use snafu::ResultExt;

/// list merge rules for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list-merge-rules")]
pub(crate) struct RepositoryListMergeRulesCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryListMergeRulesCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let mut repo_db = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *repo_db, owner, name).await?;

        let default_strategy = repo.default_strategy();
        let rules = ctx
            .db_adapter
            .merge_rules()
            .list(owner, name)
            .await
            .context(DatabaseSnafu)?;

        writeln!(
            ctx.writer,
            "Merge rules for repository {}:",
            self.repository_path
        )
        .context(IoSnafu)?;
        writeln!(ctx.writer, "- Default: '{}'", default_strategy).context(IoSnafu)?;
        for rule in rules {
            writeln!(
                ctx.writer,
                "- '{}' (base) <- '{}' (head): '{}'",
                rule.base_branch(),
                rule.head_branch(),
                rule.strategy()
            )
            .context(IoSnafu)?;
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
            "test_command_repository_list_merge_rules",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                let repo = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;

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

                db_adapter
                    .merge_rules()
                    .create(
                        MergeRule::builder()
                            .with_repository(&repo)
                            .base_branch(RuleBranch::Wildcard)
                            .head_branch(RuleBranch::Named("baz".into()))
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "list-merge-rules", "owner/name"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        Merge rules for repository owner/name:
                        - Default: 'merge'
                        - '*' (base) <- 'baz' (head): 'merge'
                        - 'foo' (base) <- 'bar' (head): 'merge'
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
