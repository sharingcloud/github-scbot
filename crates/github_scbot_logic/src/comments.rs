//! Comments logic.

use github_scbot_conf::Config;
use github_scbot_database::{
    models::{HistoryWebhookModel, IDatabaseAdapter, PullRequestModel, RepositoryModel},
    DatabaseError,
};
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{
    events::EventType,
    issues::{GhIssueCommentAction, GhIssueCommentEvent},
};
use tracing::info;

use crate::{
    commands::{AdminCommand, Command, CommandExecutor, CommandParser, CommandResult},
    pulls::PullRequestLogic,
    status::StatusLogic,
    summary::SummaryCommentSender,
    Result,
};

/// Handle an issue comment event.
pub async fn handle_issue_comment_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    event: GhIssueCommentEvent,
) -> Result<()> {
    if let GhIssueCommentAction::Created = event.action {
        let commands = CommandParser::parse_commands(config, &event.comment.body);

        match db_adapter
            .pull_request()
            .get_from_repository_path_and_number(&event.repository.full_name, event.issue.number)
            .await
        {
            Ok((mut pr_model, mut repo_model)) => {
                handle_comment_creation(
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    &mut repo_model,
                    &mut pr_model,
                    &event,
                    commands,
                )
                .await?
            }
            Err(DatabaseError::UnknownPullRequest(_, _)) => {
                // Parse admin enable
                let mut handled = false;
                for command in commands.iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        let (mut pr, sha) = PullRequestLogic::synchronize_pull_request(
                            config,
                            api_adapter,
                            db_adapter,
                            &event.repository.owner.login,
                            &event.repository.name,
                            event.issue.number,
                        )
                        .await?;

                        info!(
                            pull_request_number = event.issue.number,
                            repository_path = %event.repository.full_name,
                            message = "Manual activation on pull request",
                        );
                        let repo = pr.repository(db_adapter.repository()).await?;
                        StatusLogic::update_pull_request_status(
                            api_adapter,
                            db_adapter,
                            redis_adapter,
                            &repo,
                            &mut pr,
                            &sha,
                        )
                        .await?;

                        // Create status comment
                        SummaryCommentSender::new()
                            .create(api_adapter, db_adapter, &repo, &mut pr)
                            .await?;

                        handled = true;
                        break;
                    }
                }

                if !handled {
                    info!(
                        commands = ?commands,
                        repository_path = %event.repository.full_name,
                        pull_request_number = event.issue.number,
                        message = "Executing commands on unknown PR",
                    );
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

/// Handle comment creation.
#[allow(clippy::too_many_arguments)]
pub async fn handle_comment_creation(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    repo_model: &mut RepositoryModel,
    pr_model: &mut PullRequestModel,
    event: &GhIssueCommentEvent,
    commands: Vec<CommandResult<Command>>,
) -> Result<()> {
    let comment_author = &event.comment.user.login;
    let comment_id = event.comment.id;

    if config.server_enable_history_tracking {
        HistoryWebhookModel::builder(repo_model, pr_model)
            .username(comment_author)
            .event_key(EventType::IssueComment)
            .payload(event)
            .create(db_adapter.history_webhook())
            .await?;
    }

    info!(
        commands = ?commands,
        repository_path = %repo_model.get_path(),
        pull_request_number = pr_model.number(),
        message = "Will execute commands",
    );

    CommandExecutor::execute_commands(
        config,
        api_adapter,
        db_adapter,
        redis_adapter,
        repo_model,
        pr_model,
        comment_id,
        comment_author,
        commands,
    )
    .await?;

    Ok(())
}
