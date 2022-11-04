use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;

use crate::commands::{Command, CommandContext};

/// List known pull request for a repository
#[derive(Parser)]
pub(crate) struct PullRequestListCommand {
    /// Repository path (e.g. 'MyOrganization/my-project')
    repository_path: RepositoryPath,
}

#[async_trait(?Send)]
impl Command for PullRequestListCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        let prs = ctx.db_adapter.pull_requests().list(owner, name).await?;

        if prs.is_empty() {
            writeln!(
                ctx.writer,
                "No pull request found for repository '{}'.",
                self.repository_path
            )?;
        } else {
            for pr in prs {
                writeln!(ctx.writer, "- #{}", pr.number())?;
            }
        }

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
    async fn test_no_values() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_list()
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        let output = test_command(ctx, &["pull-requests", "list", "owner/name"]).await?;
        assert_eq!(
            output,
            "No pull request found for repository 'owner/name'.\n"
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_values() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_pull_requests().returning(|| {
            let mut mock = MockPullRequestDB::new();
            mock.expect_list()
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async {
                        Ok(vec![
                            PullRequest::builder().number(1u64).build().unwrap(),
                            PullRequest::builder().number(2u64).build().unwrap(),
                        ])
                    }
                    .boxed()
                });

            Box::new(mock)
        });

        let output = test_command(ctx, &["pull-requests", "list", "owner/name"]).await?;
        assert_eq!(
            output,
            indoc::indoc! {r#"
                - #1
                - #2
            "#}
        );

        Ok(())
    }
}
