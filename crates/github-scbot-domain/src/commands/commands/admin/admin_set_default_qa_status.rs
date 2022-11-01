use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultQaStatusCommand {
    enabled: bool,
}

impl AdminSetDefaultQaStatusCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultQaStatusCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .repositories()
            .set_default_enable_qa(ctx.repo_owner, ctx.repo_name, self.enabled)
            .await?;

        let comment = if self.enabled {
            "QA status check **enabled** for this repository."
        } else {
            "QA status check **disabled** for this repository."
        };
        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment.into()))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{MockRepositoryDB, Repository};
    use mockall::predicate;

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_enable() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_default_enable_qa()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(true),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultQaStatusCommand::new(true)
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "QA status check **enabled** for this repository.".into()
                )
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_disable() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_default_enable_qa()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(false),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultQaStatusCommand::new(false)
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "QA status check **disabled** for this repository.".into()
                )
            ]
        );

        Ok(())
    }
}
