//! Comments logic.

use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{HistoryWebhookModel, PullRequestModel, RepositoryModel},
    DatabaseError, DbPool,
};
use github_scbot_types::{
    events::EventType,
    issues::{GhIssueCommentAction, GhIssueCommentEvent},
};
use tracing::info;

use crate::{
    commands::{execute_commands, parse_commands, Command},
    pulls::synchronize_pull_request,
    status::update_pull_request_status,
    Result,
};

/// Handle an issue comment event.
pub async fn handle_issue_comment_event(
    config: Config,
    pool: DbPool,
    event: GhIssueCommentEvent,
) -> Result<()> {
    if let GhIssueCommentAction::Created = event.action {
        let conn = get_connection(&pool.clone())?;
        let commands = parse_commands(&config, &event.comment.body)?;

        match PullRequestModel::get_from_repository_path_and_number(
            &conn,
            &event.repository.full_name,
            event.issue.number,
        ) {
            Ok((mut pr_model, mut repo_model)) => {
                handle_comment_creation(
                    &config,
                    pool,
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
                for command in &commands {
                    if let Command::AdminEnable = command {
                        let (mut pr, sha) = synchronize_pull_request(
                            &config,
                            &conn,
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
                        let repo = pr.get_repository(&conn)?;
                        update_pull_request_status(&config, pool.clone(), &repo, &mut pr, &sha)
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
pub async fn handle_comment_creation(
    config: &Config,
    pool: DbPool,
    repo_model: &mut RepositoryModel,
    pr_model: &mut PullRequestModel,
    event: &GhIssueCommentEvent,
    commands: Vec<Command>,
) -> Result<()> {
    let conn = get_connection(&pool.clone())?;
    let comment_author = &event.comment.user.login;
    let comment_id = event.comment.id;

    if config.server_enable_history_tracking {
        HistoryWebhookModel::builder(&repo_model, &pr_model)
            .username(comment_author)
            .event_key(EventType::IssueComment)
            .payload(event)
            .create(&conn)?;
    }

    info!(
        commands = ?commands,
        repository_path = %repo_model.get_path(),
        pull_request_number = pr_model.get_number(),
        message = "Will execute commands",
    );

    execute_commands(
        config,
        pool.clone(),
        repo_model,
        pr_model,
        comment_id,
        comment_author,
        commands,
    )
    .await?;

    Ok(())
}
