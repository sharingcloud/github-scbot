//! Comments logic.

use github_scbot_api::comments::add_reaction_to_comment;
use github_scbot_core::Config;
use github_scbot_database::{models::RepositoryModel, DbConn};
use github_scbot_types::issues::{GHIssueCommentAction, GHIssueCommentEvent, GHReactionType};
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
/// * `conn` - Database connection
/// * `event` - GitHub Issue comment event
pub async fn handle_issue_comment_event(
    config: &Config,
    conn: &DbConn,
    event: &GHIssueCommentEvent,
) -> Result<()> {
    let repo_model = process_repository(conn, &event.repository)?;
    if let GHIssueCommentAction::Created = event.action {
        handle_comment_creation(
            config,
            conn,
            &repo_model,
            event.issue.number,
            event.comment.id,
            &event.comment.user.login,
            &event.comment.body,
        )
        .await?;
    }

    Ok(())
}

/// Handle comment creation.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `issue_number` - Issue number
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
/// * `comment_body` - Comment body
pub async fn handle_comment_creation(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    issue_number: u64,
    comment_id: u64,
    comment_author: &str,
    comment_body: &str,
) -> Result<()> {
    match get_or_fetch_pull_request(config, conn, repo_model, issue_number).await {
        Ok(mut pr_model) => {
            let status = parse_commands(
                config,
                conn,
                &repo_model,
                &mut pr_model,
                comment_id,
                comment_author,
                comment_body,
            )
            .await?;

            if matches!(status, CommandHandlingStatus::Handled) {
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GHReactionType::Eyes,
                )
                .await?;
            }
        }
        Err(e) => error!("{}", e),
    }

    Ok(())
}
