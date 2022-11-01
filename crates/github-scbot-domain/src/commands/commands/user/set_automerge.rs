use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetAutomergeCommand {
    enabled: bool,
}

impl SetAutomergeCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn new_enabled() -> Self {
        Self { enabled: true }
    }

    pub fn new_disabled() -> Self {
        Self { enabled: false }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetAutomergeCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .pull_requests()
            .set_automerge(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.enabled)
            .await?;

        let status_text = if self.enabled { "enabled" } else { "disabled" };
        let comment = format!(
            "Automerge **{}** by **{}**.",
            status_text, ctx.comment_author
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

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_enabled() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_automerge()
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

        let cmd = SetAutomergeCommand::new_enabled();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Automerge **enabled** by **me**.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_disabled() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .expect_pull_requests()
            .times(1)
            .returning(|| {
                let mut mock = MockPullRequestDB::new();
                mock.expect_set_automerge()
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

        let cmd = SetAutomergeCommand::new_disabled();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Automerge **disabled** by **me**.".into())
            ]
        );

        Ok(())
    }
}
