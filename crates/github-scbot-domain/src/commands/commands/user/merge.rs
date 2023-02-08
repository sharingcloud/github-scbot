use github_scbot_core::types::{issues::GhReactionType, labels::StepLabel, pulls::GhMergeStrategy};

use async_trait::async_trait;
use tracing::error;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    pulls::PullRequestLogic,
    use_cases::status::{BuildPullRequestStatusUseCase, DetermineAutomaticStepUseCase},
    Result,
};

pub struct MergeCommand {
    strategy: Option<GhMergeStrategy>,
}

impl MergeCommand {
    pub fn new(strategy: Option<GhMergeStrategy>) -> Self {
        Self { strategy }
    }

    pub fn new_default_strategy() -> Self {
        Self { strategy: None }
    }
}

#[async_trait(?Send)]
impl BotCommand for MergeCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        let pr_status = BuildPullRequestStatusUseCase {
            api_service: ctx.api_adapter,
            db_service: ctx.db_adapter,
            pr_number: ctx.pr_number,
            repo_name: ctx.repo_name,
            repo_owner: ctx.repo_owner,
            upstream_pr: ctx.upstream_pr,
        }
        .run()
        .await?;

        let step = DetermineAutomaticStepUseCase {
            pr_status: &pr_status,
        }
        .run();

        let commit_title = PullRequestLogic::get_merge_commit_title(ctx.upstream_pr);
        let mut actions = vec![];

        // Use step to determine merge possibility
        if step == StepLabel::AwaitingMerge {
            let strategy = self.strategy.unwrap_or(pr_status.merge_strategy);
            if let Err(e) = ctx
                .api_adapter
                .pulls_merge(
                    ctx.repo_owner,
                    ctx.repo_name,
                    ctx.pr_number,
                    &commit_title,
                    "",
                    strategy,
                )
                .await
            {
                error!(
                    owner = %ctx.repo_owner,
                    name = %ctx.repo_name,
                    pr_number = ctx.pr_number,
                    error = %e,
                    message = "Error while merging pull request"
                );

                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(
                    "Error while merging this pull request.".to_string(),
                ));
            } else {
                actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
                actions.push(ResultAction::PostComment(format!(
                    "Pull request successfully merged by **{}**! (strategy: **{}**)",
                    ctx.comment_author, strategy
                )));
            }
        } else {
            actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
            actions.push(ResultAction::PostComment(
                "Pull request is not ready to merge.".into(),
            ));
        }

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_actions(actions)
            .build())
    }
}
