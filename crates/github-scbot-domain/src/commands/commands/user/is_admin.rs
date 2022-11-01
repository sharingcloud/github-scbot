use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    auth::AuthLogic,
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct IsAdminCommand;

impl IsAdminCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl BotCommand for IsAdminCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let known_admins = AuthLogic::list_known_admin_usernames(ctx.db_adapter).await?;
        let status = AuthLogic::is_admin(ctx.comment_author, &known_admins);
        let reaction_type = if status {
            GhReactionType::PlusOne
        } else {
            GhReactionType::MinusOne
        };

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(reaction_type))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use github_scbot_database::{Account, MockAccountDB};

    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_not_admin() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_accounts().times(1).returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins()
                .returning(|| async { Ok(vec![]) }.boxed());

            Box::new(mock)
        });

        let cmd = IsAdminCommand::new();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::MinusOne)]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_admin() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_accounts().times(1).returning(|| {
            let mut mock = MockAccountDB::new();
            mock.expect_list_admins().returning(|| {
                async {
                    Ok(vec![Account::builder()
                        .username("me")
                        .is_admin(true)
                        .build()
                        .unwrap()])
                }
                .boxed()
            });

            Box::new(mock)
        });

        let cmd = IsAdminCommand::new();
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::PlusOne)]
        );

        Ok(())
    }
}
