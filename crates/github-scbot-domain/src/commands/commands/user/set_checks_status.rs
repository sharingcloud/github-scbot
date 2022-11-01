use github_scbot_core::types::{issues::GhReactionType, status::CheckStatus};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetChecksStatusCommand {
    status: CheckStatus,
}

impl SetChecksStatusCommand {
    pub fn new_skip_or_wait(status: bool) -> Self {
        Self {
            status: if status {
                CheckStatus::Skipped
            } else {
                CheckStatus::Waiting
            },
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetChecksStatusCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let value = !matches!(self.status, CheckStatus::Skipped);

        ctx.db_adapter
            .pull_requests()
            .set_checks_enabled(ctx.repo_owner, ctx.repo_name, ctx.pr_number, value)
            .await?;

        let comment = format!(
            "Check status is marked as **{}** by **{}**.",
            self.status.to_str(),
            ctx.comment_author
        );

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::MockPullRequestDB;
    use github_scbot_database::PullRequest;
    use mockall::predicate;

    use super::*;
    use crate::commands::CommandContextTest;

    #[actix_rt::test]
    async fn test_skip() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Skip.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_checks_enabled()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(false),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetChecksStatusCommand::new_skip_or_wait(true);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Check status is marked as **skipped** by **me**.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_reset() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        // Reset.
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_checks_enabled()
                    .with(
                        predicate::eq("owner"),
                        predicate::eq("name"),
                        predicate::eq(1),
                        predicate::eq(true),
                    )
                    .returning(|_, _, _, _| {
                        async { Ok(PullRequest::builder().build().unwrap()) }.boxed()
                    });
                Box::new(mock)
            });

        let cmd = SetChecksStatusCommand::new_skip_or_wait(false);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Check status is marked as **waiting** by **me**.".into()
                )
            ]
        );

        Ok(())
    }
}
