//! Comments logic.

use github_scbot_api::comments::add_reaction_to_comment;
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{HistoryWebhookModel, PullRequestModel, RepositoryModel},
    DatabaseError, DbPool,
};
use github_scbot_types::{
    events::EventType,
    issues::{GhIssueCommentAction, GhIssueCommentEvent, GhReactionType},
};
use tracing::info;

use crate::{
    commands::{execute_commands, parse_commands, Command, CommandHandlingStatus},
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
            Ok((mut pr_model, repo_model)) => {
                handle_comment_creation(&config, pool, &repo_model, &mut pr_model, &event, commands)
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
                            "Enabling PR #{}, repository {}",
                            event.issue.number, event.repository.full_name
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
                        "Trying to execute commands {:?} from comment on repository {}, unknown PR #{}",
                        commands, event.repository.full_name, event.issue.number
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
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    event: &GhIssueCommentEvent,
    commands: Vec<Command>,
) -> Result<()> {
    let conn = get_connection(&pool.clone())?;
    let comment_author = &event.comment.user.login;
    let comment_id = event.comment.id;

    HistoryWebhookModel::builder(&repo_model, &pr_model)
        .username(comment_author)
        .event_key(EventType::IssueComment)
        .payload(event)
        .create(&conn)?;

    info!(
        "Will execute commands {:?} on repository {}, PR #{}",
        commands,
        repo_model.get_path(),
        pr_model.get_number()
    );

    let statuses = execute_commands(
        config,
        pool.clone(),
        repo_model,
        pr_model,
        comment_id,
        comment_author,
        commands,
    )
    .await?;

    for status in statuses {
        match status {
            CommandHandlingStatus::Handled => {
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GhReactionType::Eyes,
                )
                .await?;
                break;
            }
            CommandHandlingStatus::Denied => {
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GhReactionType::MinusOne,
                )
                .await?;
                break;
            }
            CommandHandlingStatus::Ignored => (),
        }
    }

    Ok(())
}
