use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    pulls::PullRequestLogic,
    Result,
};

pub struct AdminSyncCommand;

impl AdminSyncCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSyncCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        PullRequestLogic::synchronize_pull_request(
            ctx.config,
            ctx.db_adapter,
            ctx.repo_owner,
            ctx.repo_name,
            ctx.pr_number,
        )
        .await?;

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockPullRequestDB, MockRepositoryDB, PullRequest, Repository};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_command() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter.expect_repositories().times(1).returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_get()
                .times(1)
                .with(predicate::eq("owner"), predicate::eq("name"))
                .returning(|_, _| {
                    async { Ok(Some(Repository::builder().build().unwrap())) }.boxed()
                });

            Box::new(mock)
        });
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

        let result = AdminSyncCommand::new().handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::Eyes)]
        );

        Ok(())
    }
}
