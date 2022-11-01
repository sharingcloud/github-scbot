use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct AdminSetDefaultReviewersCommand {
    count: u64,
}

impl AdminSetDefaultReviewersCommand {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

#[async_trait(?Send)]
impl BotCommand for AdminSetDefaultReviewersCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_adapter
            .repositories()
            .set_default_needed_reviewers_count(ctx.repo_owner, ctx.repo_name, self.count)
            .await?;

        let comment = format!(
            "Needed reviewers count set to **{}** for this repository.",
            self.count
        );
        Ok(CommandExecutionResult::builder()
            .with_status_update(false)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
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
    async fn test_command() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter.expect_repositories().returning(|| {
            let mut mock = MockRepositoryDB::new();
            mock.expect_set_default_needed_reviewers_count()
                .with(
                    predicate::eq("owner"),
                    predicate::eq("name"),
                    predicate::eq(0),
                )
                .returning(|_, _, _| async { Ok(Repository::builder().build().unwrap()) }.boxed());

            Box::new(mock)
        });

        let result = AdminSetDefaultReviewersCommand::new(0)
            .handle(&ctx.as_context())
            .await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "Needed reviewers count set to **0** for this repository.".into()
                )
            ]
        );

        Ok(())
    }
}
