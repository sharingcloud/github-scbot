use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultChecksStatusCommand {
    enabled: bool,
}

impl AdminSetDefaultChecksStatusCommand {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultChecksStatusCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .repositories()
            .set_default_enable_checks(ctx.repo_owner, ctx.repo_name, self.enabled)
            .await?;

        let comment = if self.enabled {
            "Checks **enabled** for this repository."
        } else {
            "Checks **disabled** for this repository."
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
            mock.expect_set_default_enable_checks()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(true),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultChecksStatusCommand::new(true)
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Checks **enabled** for this repository.".into())
            ]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_disable() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_default_enable_checks()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(false),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultChecksStatusCommand::new(false)
            .handle(&ctx.as_context())
            .await?;
        assert!(result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("Checks **disabled** for this repository.".into())
            ]
        );

        Ok(())
    }
}
