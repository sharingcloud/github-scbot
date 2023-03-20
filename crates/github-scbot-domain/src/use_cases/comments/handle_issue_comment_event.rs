use github_scbot_config::Config;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{
    types::{GhIssueCommentAction, GhIssueCommentEvent},
    ApiService,
};
use github_scbot_lock_interface::LockService;
use tracing::info;

use crate::{
    commands::{AdminCommand, Command, CommandContext, CommandExecutor, CommandParser},
    use_cases::{pulls::SynchronizePullRequestUseCase, status::UpdatePullRequestStatusUseCase},
    Result,
};

pub struct HandleIssueCommentEventUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
    pub lock_service: &'a dyn LockService,
}

impl<'a> HandleIssueCommentEventUseCase<'a> {
    #[tracing::instrument(skip(self), fields(
        action = ?event.action,
        repo_owner = event.repository.owner.login,
        repo_name = event.repository.name,
        number = event.issue.number
    ))]
    pub async fn run(&self, event: GhIssueCommentEvent) -> Result<()> {
        if let GhIssueCommentAction::Created = event.action {
            self.run_created_comment(event).await
        } else {
            Ok(())
        }
    }

    async fn run_created_comment(&self, event: GhIssueCommentEvent) -> Result<()> {
        let repo_owner = &event.repository.owner.login;
        let repo_name = &event.repository.name;
        let pr_number = event.issue.number;
        let pr_handle: &PullRequestHandle =
            &(repo_owner.as_str(), repo_name.as_str(), pr_number).into();

        let commands = CommandParser::parse_commands(self.config, &event.comment.body);
        match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_) => {
                let upstream_pr = self
                    .api_service
                    .pulls_get(repo_owner, repo_name, pr_number)
                    .await?;

                let mut ctx = CommandContext {
                    config: self.config,
                    api_service: self.api_service,
                    db_service: self.db_service,
                    lock_service: self.lock_service,
                    repo_owner,
                    repo_name,
                    pr_number,
                    upstream_pr: &upstream_pr,
                    comment_id: event.comment.id,
                    comment_author: &event.comment.user.login,
                };

                CommandExecutor::execute_commands(&mut ctx, commands).await?;
            }
            None => {
                // Parse admin enable
                let mut handled = false;
                for command in commands.iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        let upstream_pr = self
                            .api_service
                            .pulls_get(repo_owner, repo_name, pr_number)
                            .await?;

                        SynchronizePullRequestUseCase {
                            config: self.config,
                            db_service: self.db_service,
                        }
                        .run(pr_handle)
                        .await?;

                        info!(
                            pull_request_number = event.issue.number,
                            repository_path = %event.repository.full_name,
                            message = "Manual activation on pull request",
                        );

                        UpdatePullRequestStatusUseCase {
                            api_service: self.api_service,
                            db_service: self.db_service,
                            lock_service: self.lock_service,
                        }
                        .run(pr_handle, &upstream_pr)
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
        }

        Ok(())
    }
}
