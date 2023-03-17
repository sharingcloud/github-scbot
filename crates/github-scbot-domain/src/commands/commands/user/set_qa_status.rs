use github_scbot_core::types::{issues::GhReactionType, status::QaStatus};

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    Result,
};

pub struct SetQaStatusCommand {
    status: QaStatus,
}

impl SetQaStatusCommand {
    pub fn new(status: QaStatus) -> Self {
        Self { status }
    }

    pub fn new_skip_or_wait(status: bool) -> Self {
        Self {
            status: if status {
                QaStatus::Skipped
            } else {
                QaStatus::Waiting
            },
        }
    }

    pub fn new_pass_or_fail(status: Option<bool>) -> Self {
        Self {
            status: match status {
                None => QaStatus::Waiting,
                Some(true) => QaStatus::Pass,
                Some(false) => QaStatus::Fail,
            },
        }
    }

    fn _create_status_message(&self, ctx: &mut CommandContext) -> String {
        let status = match self.status {
            QaStatus::Fail => "failed",
            QaStatus::Pass => "passed",
            QaStatus::Skipped => "skipped",
            QaStatus::Waiting => "waiting",
        };

        format!(
            "QA status is marked as **{}** by **{}**.",
            status, ctx.comment_author
        )
    }
}

#[async_trait(?Send)]
impl BotCommand for SetQaStatusCommand {
    async fn handle(&self, ctx: &mut CommandContext) -> Result<CommandExecutionResult> {
        ctx.db_service
            .pull_requests_set_qa_status(ctx.repo_owner, ctx.repo_name, ctx.pr_number, self.status)
            .await?;

        let comment = self._create_status_message(ctx);

        Ok(CommandExecutionResult::builder()
            .with_status_update(true)
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(comment))
            .build())
    }
}
