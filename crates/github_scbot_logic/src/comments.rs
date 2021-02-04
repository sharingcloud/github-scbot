//! Comments logic.

use github_scbot_api::comments::add_reaction_to_comment;
use github_scbot_database::{models::RepositoryModel, DbConn};
use github_scbot_types::issues::{GHIssueCommentAction, GHIssueCommentEvent, GHReactionType};

use crate::{
    commands::{parse_comment, CommandHandlingStatus},
    database::{get_or_fetch_pull_request, process_repository},
    Result,
};

/// Handle an issue comment event.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `event` - GitHub Issue comment event
pub async fn handle_issue_comment_event(conn: &DbConn, event: &GHIssueCommentEvent) -> Result<()> {
    let repo_model = process_repository(conn, &event.repository)?;
    if let GHIssueCommentAction::Created = event.action {
        handle_comment_creation(
            conn,
            &repo_model,
            event.issue.number,
            &event.comment.body,
            event.comment.id,
            &event.issue.user.login,
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
/// * `issue_body` - Issue body
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
pub async fn handle_comment_creation(
    conn: &DbConn,
    repo_model: &RepositoryModel,
    issue_number: u64,
    issue_body: &str,
    comment_id: u64,
    comment_author: &str,
) -> Result<()> {
    let mut pr_model = get_or_fetch_pull_request(conn, repo_model, issue_number).await?;

    let status =
        parse_comment(conn, &repo_model, &mut pr_model, comment_author, issue_body).await?;

    if matches!(status, CommandHandlingStatus::Handled) {
        add_reaction_to_comment(
            &repo_model.owner,
            &repo_model.name,
            comment_id,
            GHReactionType::Eyes,
        )
        .await?;
    }

    Ok(())
}
