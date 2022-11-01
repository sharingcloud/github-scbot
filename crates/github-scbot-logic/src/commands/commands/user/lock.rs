use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct LockCommand {
    locked: bool,
    reason: Option<String>,
}

impl LockCommand {
    pub fn new(locked: bool, reason: Option<String>) -> Self {
        Self { locked, reason }
    }
}

#[async_trait(?Send)]
impl BotCommand for LockCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let status_text = if self.locked { "locked" } else { "unlocked" };
        ctx.db_adapter
            .pull_requests()
            .set_locked(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.locked)
            .await?;

        let mut comment = format!(
            "Pull request **{}** by **{}**.",
            status_text, ctx.comment_author
        );
        if let Some(reason) = &self.reason {
            comment = format!("{}\n**Reason**: {}.", comment, reason);
        }

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

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_lock() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_locked()
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

        let cmd = LockCommand::new(true, None);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Pull request **locked** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_lock_reason() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_locked()
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

        let cmd = LockCommand::new(true, Some("because!".into()));
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Pull request **locked** by **me**.\n**Reason**: because!.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unlock() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_locked()
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

        let cmd = LockCommand::new(false, None);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Pull request **unlocked** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_unlock_reason() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_locked()
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

        let cmd = LockCommand::new(false, Some("because!".into()));
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Pull request **unlocked** by **me**.\n**Reason**: because!.".into()
                )
            ]
        );

        Ok(())
    }
}
