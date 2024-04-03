use async_trait::async_trait;
use prbot_ghapi_interface::types::GhReactionType;
use prbot_models::MergeStrategy;
use shaku::HasComponent;

use crate::{
    bot_commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::{
        pulls::{
            try_merge_pull_request_from_status::TryMergePullRequestState,
            TryMergePullRequestFromStatusInterface,
        },
        status::BuildPullRequestStatusInterface,
    },
    Result,
};

pub struct MergeCommand {
    strategy: Option<MergeStrategy>,
}

impl MergeCommand {
    pub fn new(strategy: Option<MergeStrategy>) -> Self {
        Self { strategy }
    }

    pub fn new_default_strategy() -> Self {
        Self { strategy: None }
    }
}

#[async_trait]
impl BotCommand for MergeCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let build_pr_status: &dyn BuildPullRequestStatusInterface = ctx.core_module.resolve_ref();
        let pr_status = build_pr_status
            .run(&ctx.as_core_context(), &ctx.pr_handle(), ctx.upstream_pr)
            .await?;

        let try_merge: &dyn TryMergePullRequestFromStatusInterface = ctx.core_module.resolve_ref();
        let mut actions = vec![];

        match try_merge
            .run(
                &ctx.as_core_context(),
                &ctx.pr_handle(),
                ctx.upstream_pr,
                &pr_status,
                self.strategy,
            )
            .await?
        {
            TryMergePullRequestState::NotReady => {
                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(
                    "Pull request is not ready to merge.".into(),
                ));
            }
            TryMergePullRequestState::AlreadyLocked => {
                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(
                    "Pull request could not be merged because of a lock error.".into(),
                ));
            }
            TryMergePullRequestState::Success(strategy) => {
                actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
                actions.push(ResultAction::PostComment(format!(
                    "Pull request successfully merged by **{}**! (strategy: **{}**)",
                    ctx.comment_author, strategy
                )));
            }
            TryMergePullRequestState::Error => {
                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(
                    "Error while merging this pull request.".to_string(),
                ));
            }
        }

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_actions(actions)
            .build())
    }
}
