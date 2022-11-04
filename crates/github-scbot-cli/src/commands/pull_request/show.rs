use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

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
            "Accessing pull request #{} on repository '{}':",
            self.number, self.repository_path
        )?;
        writeln!(ctx.writer, "{:#?}", pr)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockPullRequestDB, PullRequest};
    use mockall::predicate;

    use crate::testutils::{test_command, CommandContextTest};
    use crate::Result;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_get()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(1),
                )
                .returning(|_, _, _| {
                    async { Ok(Some(PullRequest::builder().build().unwrap())) }.boxed()
                });

            Box::new(mock)
        });

        let output = test_command(ctx, &["pull-requests", "show", "owner/name", "1"]).await?;
        assert_eq!(
            output,
            indoc::indoc! {r#"
                Accessing pull request #1 on repository 'owner/name':
                PullRequest {
                    id: 0,
                    repository_id: 0,
                    number: 0,
                    qa_status: Waiting,
                    needed_reviewers_count: 0,
                    status_comment_id: 0,
                    checks_enabled: false,
                    automerge: false,
                    locked: false,
                    strategy_override: None,
                }
            "#}
        );

        Ok(())
    }
}
