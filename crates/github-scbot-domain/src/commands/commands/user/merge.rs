use async_trait::async_trait;
use github_scbot_domain_models::{MergeStrategy, StepLabel};
use github_scbot_ghapi_interface::types::GhReactionType;
use tracing::error;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::{
        checks::DetermineChecksStatusUseCase,
        pulls::{MergePullRequestUseCase, MergePullRequestUseCaseInterface},
        status::{
            BuildPullRequestStatusUseCase, BuildPullRequestStatusUseCaseInterface, StepLabelChooser,
        },
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

#[async_trait(?Send)]
impl BotCommand for MergeCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let pr_status = BuildPullRequestStatusUseCase {
            api_service: ctx.api_service,
            db_service: ctx.db_service,
            determine_checks_status: &DetermineChecksStatusUseCase {
                api_service: ctx.api_service,
            },
        }
        .run(&ctx.pr_handle(), ctx.upstream_pr)
        .await?;

        let step = StepLabelChooser::default().choose_from_status(&pr_status);

        let mut actions = vec![];

        // Use step to determine merge possibility
        if step == StepLabel::AwaitingMerge {
            let strategy = self.strategy.unwrap_or(pr_status.merge_strategy);
            let merge_result = MergePullRequestUseCase {
                api_service: ctx.api_service,
            }
            .run(&ctx.pr_handle(), strategy, ctx.upstream_pr)
            .await;

            match merge_result {
                Ok(()) => {
                    actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
                    actions.push(ResultAction::PostComment(format!(
                        "Pull request successfully merged by **{}**! (strategy: **{}**)",
                        ctx.comment_author, strategy
                    )));
                }
                Err(e) => {
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
                }
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
