use github_scbot_core::config::Config;
use github_scbot_database_interface::DbService;
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
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub event: GhIssueCommentEvent,
}

impl<'a> HandleIssueCommentEventUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        if let GhIssueCommentAction::Created = self.event.action {
            self.run_created_comment().await
        } else {
            Ok(())
        }
    }

    async fn run_created_comment(&mut self) -> Result<()> {
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;
        let pr_number = self.event.issue.number;

        let commands = CommandParser::parse_commands(self.config, &self.event.comment.body);
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
                    comment_id: self.event.comment.id,
                    comment_author: &self.event.comment.user.login,
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
                            repo_owner,
                            repo_name,
                            pr_number,
                        }
                        .run()
                        .await?;

                        info!(
                            pull_request_number = self.event.issue.number,
                            repository_path = %self.event.repository.full_name,
                            message = "Manual activation on pull request",
                        );

                        UpdatePullRequestStatusUseCase {
                            api_service: self.api_service,
                            db_service: self.db_service,
                            lock_service: self.lock_service,
                            repo_name,
                            repo_owner,
                            pr_number,
                            upstream_pr: &upstream_pr,
                        }
                        .run()
                        .await?;

                        handled = true;
                        break;
                    }
                }

                if !handled {
                    info!(
                        commands = ?commands,
                        repository_path = %self.event.repository.full_name,
                        pull_request_number = self.event.issue.number,
                        message = "Executing commands on unknown PR",
                    );
                }
            }
        }

        Ok(())
    }
}
