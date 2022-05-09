use std::io::Write;

use crate::Result;
use argh::FromArgs;
use async_trait::async_trait;
use github_scbot_types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};
use crate::errors::{DatabaseSnafu, IoSnafu};
use snafu::ResultExt;

/// list known pull request for a repository.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub(crate) struct PullRequestListCommand {
    /// repository path (e.g. 'MyOrganization/my-project')
    #[argh(positional)]
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for PullRequestListCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let prs = ctx
            .db_adapter
            .pull_requests()
            .list(owner, name)
            .await
            .context(DatabaseSnafu)?;

        if prs.is_empty() {
            writeln!(
                ctx.writer,
                "No PR found from repository '{}'.",
                self.repository_path
            )
            .context(IoSnafu)?;
        } else {
            for pr in prs {
                writeln!(ctx.writer, "- #{}", pr.number()).context(IoSnafu)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_database2::{
        use_temporary_db, DbService, DbServiceImplPool, PullRequest, Repository,
    };
    use github_scbot_ghapi::adapter::MockApiService;
    use github_scbot_redis::MockRedisService;

    use crate::testutils::test_command;

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(
            config,
            "test_command_pull_request_list",
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
                    &["pull-requests", "list", "owner/name"],
                )
                .await?;

                assert_eq!(output, "No PR found from repository 'owner/name'.\n");

                let db_adapter = DbServiceImplPool::new(pool.clone());
                db_adapter
                    .pull_requests()
                    .create(
                        PullRequest::builder()
                            .with_repository(&repo)
                            .number(1u64)
                            .build()?,
                    )
                    .await?;
                db_adapter
                    .pull_requests()
                    .create(
                        PullRequest::builder()
                            .with_repository(&repo)
                            .number(2u64)
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config,
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["pull-requests", "list", "owner/name"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        - #1
                        - #2
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
