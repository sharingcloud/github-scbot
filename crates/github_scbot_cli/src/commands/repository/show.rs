use std::io::Write;

use crate::errors::IoSnafu;
use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::repository::RepositoryPath;
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// show repository info.
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub(crate) struct RepositoryShowCommand {
    /// repository path (e.g. `MyOrganization/my-project`).
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for RepositoryShowCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let mut pr_repo = ctx.db_adapter.repositories();
        let repo = CliDbExt::get_existing_repository(&mut *pr_repo, owner, name).await?;

        writeln!(ctx.writer, "Accessing repository {}", self.repository_path).context(IoSnafu)?;
        writeln!(ctx.writer, "{:#?}", repo).context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{use_temporary_db, DbService, DbServiceImplPool, Repository};
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_repository_show",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["repositories", "show", "owner/name"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        Accessing repository owner/name
                        Repository {
                            id: 1,
                            owner: "owner",
                            name: "name",
                            manual_interaction: false,
                            pr_title_validation_regex: "",
                            default_strategy: Merge,
                            default_needed_reviewers_count: 0,
                            default_automerge: false,
                            default_enable_qa: false,
                            default_enable_checks: true,
                        }
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
