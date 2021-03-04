//! Comments logic.

use github_scbot_api::comments::add_reaction_to_comment;
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{HistoryWebhookModel, RepositoryModel},
    DbPool,
};
use github_scbot_types::{
    events::EventType,
    issues::{GHIssueCommentAction, GHIssueCommentEvent, GHReactionType},
};
use tracing::error;

use crate::{
    commands::{parse_commands, CommandHandlingStatus},
    database::{get_or_fetch_pull_request, process_repository},
    Result,
};

/// Handle an issue comment event.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `pool` - Database pool
/// * `event` - GitHub Issue comment event
pub async fn handle_issue_comment_event(
    config: Config,
    pool: DbPool,
    event: GHIssueCommentEvent,
) -> Result<()> {
    let repo_model =
        process_repository(config.clone(), pool.clone(), event.repository.clone()).await?;
    if let GHIssueCommentAction::Created = event.action {
        handle_comment_creation(&config, pool, &repo_model, &event).await?;
    }

    Ok(())
}

/// Handle comment creation.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `pool` - Database pool
/// * `repo_model` - Repository model
/// * `event` - GitHub Issue comment event
pub async fn handle_comment_creation(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    event: &GHIssueCommentEvent,
) -> Result<()> {
    let conn = get_connection(&pool.clone())?;
    let issue_number = event.issue.number;
    let comment_author = &event.comment.user.login;
    let comment_body = &event.comment.body;
    let comment_id = event.comment.id;

    match get_or_fetch_pull_request(config, &conn, repo_model, issue_number).await {
        Ok(mut pr_model) => {
            HistoryWebhookModel::builder(&repo_model, &pr_model)
                .username(comment_author)
                .event_key(EventType::IssueComment)
                .payload(event)
                .create(&conn)?;

            let status = parse_commands(
                config,
                pool,
                &repo_model,
                &mut pr_model,
                comment_id,
                comment_author,
                comment_body,
            )
            .await?;

            match status {
                CommandHandlingStatus::Handled => {
                    add_reaction_to_comment(
                        config,
                        &repo_model.owner,
                        &repo_model.name,
                        comment_id,
                        GHReactionType::Eyes,
                    )
                    .await?
                }
                CommandHandlingStatus::Denied => {
                    add_reaction_to_comment(
                        config,
                        &repo_model.owner,
                        &repo_model.name,
                        comment_id,
                        GHReactionType::MinusOne,
                    )
                    .await?
                }
                CommandHandlingStatus::Ignored => (),
            }
        }
        Err(e) => error!("{}", e),
    }

    Ok(())
}
