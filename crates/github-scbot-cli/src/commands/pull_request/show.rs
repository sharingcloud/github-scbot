use std::io::Write;

use crate::errors::IoSnafu;
use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use snafu::ResultExt;

use crate::{
    commands::{Command, CommandContext},
    utils::CliDbExt,
};

/// Show pull request info
#[derive(Parser)]
pub(crate) struct PullRequestShowCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,

    /// Pull request number
    number: u64,
}

#[async_trait(?Send)]
impl Command for PullRequestShowCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();
        let pr = CliDbExt::get_existing_pull_request(
            &mut *ctx.db_adapter.pull_requests(),
            owner,
            name,
            self.number,
        )
        .await?;

        writeln!(
            ctx.writer,
            "Accessing pull request #{} on repository {}",
            self.number, self.repository_path
        )
        .context(IoSnafu)?;
        writeln!(ctx.writer, "{:#?}", pr).context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::config::Config;
    use github_scbot_database::{
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
            "test_command_pull_request_show",
            |config, pool| async move {
                let db_adapter = DbServiceImplPool::new(pool.clone());
                let repo = db_adapter
                    .repositories()
                    .create(Repository::builder().owner("owner").name("name").build()?)
                    .await?;

                db_adapter
                    .pull_requests()
                    .create(
                        PullRequest::builder()
                            .with_repository(&repo)
                            .number(1u64)
                            .build()?,
                    )
                    .await?;

                let output = test_command(
                    config.clone(),
                    Box::new(db_adapter),
                    Box::new(MockApiService::new()),
                    Box::new(MockRedisService::new()),
                    &["pull-requests", "show", "owner/name", "1"],
                )
                .await?;

                assert_eq!(
                    output,
                    indoc::indoc! {r#"
                        Accessing pull request #1 on repository owner/name
                        PullRequest {
                            id: 1,
                            repository_id: 1,
                            number: 1,
                            qa_status: Skipped,
                            needed_reviewers_count: 0,
                            status_comment_id: 0,
                            checks_enabled: true,
                            automerge: false,
                            locked: false,
                            strategy_override: None,
                        }
                    "#}
                );

                Ok(())
            },
        )
        .await;
    }
}
