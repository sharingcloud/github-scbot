//! Comments logic.

use github_scbot_conf::Config;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;
use github_scbot_types::{
    issues::{GhIssueCommentAction, GhIssueCommentEvent},
    pulls::GhPullRequest,
};
use tracing::info;

use crate::{
    commands::{AdminCommand, Command, CommandExecutor, CommandParser, CommandResult},
    pulls::PullRequestLogic,
    status::StatusLogic,
    Result,
};

/// Handle an issue comment event.
pub async fn handle_issue_comment_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhIssueCommentEvent,
) -> Result<()> {
    if let GhIssueCommentAction::Created = event.action {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.issue.number;

        let commands = CommandParser::parse_commands(config, &event.comment.body);
        match db_adapter
            .pull_requests()
            .get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_) => {
                let upstream_pr = api_adapter
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                handle_comment_creation(
                    config,
                    api_adapter,
                    db_adapter,
                    redis_adapter,
                    repo_owner,
                    repo_name,
                    pr_number,
                    &upstream_pr,
                    &event,
                    commands,
                )
                .await?
            }
            None => {
                // Parse admin enable
                let mut handled = false;
                for command in commands.iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        let upstream_pr = api_adapter
                            .pulls_get(repo_owner, repo_name, pr_number)
                            .await?;

                        PullRequestLogic::synchronize_pull_request(
                            config, db_adapter, repo_owner, repo_name, pr_number,
                        )
                        .await?;

                        info!(
                            pull_request_number = event.issue.number,
                            repository_path = %event.repository.full_name,
                            message = "Manual activation on pull request",
                        );

                        StatusLogic::update_pull_request_status(
                            api_adapter,
                            db_adapter,
                            redis_adapter,
                            repo_owner,
                            repo_name,
                            pr_number,
                            &upstream_pr,
                        )
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
        };
    }

    Ok(())
}

/// Handle comment creation.
#[allow(clippy::too_many_arguments)]
pub async fn handle_comment_creation(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    upstream_pr: &GhPullRequest,
    event: &GhIssueCommentEvent,
    commands: Vec<CommandResult<Command>>,
) -> Result<()> {
    let comment_author = &event.comment.user.login;
    let comment_id = event.comment.id;

    CommandExecutor::execute_commands(
        config,
        api_adapter,
        db_adapter,
        redis_adapter,
        repo_owner,
        repo_name,
        pr_number,
        upstream_pr,
        comment_id,
        comment_author,
        commands,
    )
    .await?;

    Ok(())
}
