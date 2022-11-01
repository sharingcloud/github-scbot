use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetLabelsCommand {
    added: Vec<String>,
    removed: Vec<String>,
}

impl SetLabelsCommand {
    pub fn new_added(added: Vec<String>) -> Self {
        Self {
            added,
            removed: vec![],
        }
    }

    pub fn new_removed(removed: Vec<String>) -> Self {
        Self {
            added: vec![],
            removed,
        }
    }
}

#[async_trait(?Send)]
impl BotCommand for SetLabelsCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        if !self.added.is_empty() {
            ctx.api_adapter
                .issue_labels_add(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.added)
                .await?;
        }

        if !self.removed.is_empty() {
            ctx.api_adapter
                .issue_labels_remove(ctx.repo_owner, ctx.repo_name, ctx.pr_number, &self.removed)
                .await?;
        }

        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::CommandContextTest;

    use super::*;

    #[actix_rt::test]
    async fn test_add() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.api_adapter
            .expect_issue_labels_add()
            .times(1)
            .withf(|owner, name, pr_number, labels| {
                owner == "owner" && name == "name" && *pr_number == 1u64 && labels == ["one", "two"]
            })
            .returning(|_, _, _, _| Ok(()));

        let cmd = SetLabelsCommand::new_added(vec!["one".into(), "two".into()]);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::Eyes),]
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn test_remove() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.api_adapter
            .expect_issue_labels_remove()
            .times(1)
            .withf(|owner, name, pr_number, labels| {
                owner == "owner" && name == "name" && *pr_number == 1u64 && labels == ["one", "two"]
            })
            .returning(|_, _, _, _| Ok(()));

        let cmd = SetLabelsCommand::new_removed(vec!["one".into(), "two".into()]);
        let result = cmd.handle(&ctx.as_context()).await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![ResultAction::AddReaction(GhReactionType::Eyes),]
        );

        Ok(())
    }
}
