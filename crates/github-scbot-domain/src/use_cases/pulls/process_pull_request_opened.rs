use github_scbot_core::{config::Config, types::pulls::GhPullRequestEvent};
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{PullRequest, Repository};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use crate::{
    commands::{AdminCommand, Command, CommandContext, CommandExecutor, CommandParser},
    use_cases::{comments::PostWelcomeCommentUseCase, status::UpdatePullRequestStatusUseCase},
    Result,
};

use super::GetOrCreateRepositoryUseCase;

/// Pull request opened status.
#[derive(Debug, PartialEq, Eq)]
pub enum PullRequestOpenedStatus {
    /// Pull request is already created.
    AlreadyCreated,
    /// Pull request is created.
    Created,
    /// Pull request is ignored.
    Ignored,
}

pub struct ProcessPullRequestOpenedUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub lock_service: &'a dyn LockService,
    pub event: GhPullRequestEvent,
}

impl<'a> ProcessPullRequestOpenedUseCase<'a> {
    #[tracing::instrument(
        skip_all,
        fields(
            action = ?self.event.action,
            pr_number = self.event.number,
            repository_path = %self.event.repository.full_name,
            username = %self.event.pull_request.user.login
        )
    )]
    pub async fn run(&mut self) -> Result<PullRequestOpenedStatus> {
        // Get or create repository
        let repo_owner = &self.event.repository.owner.login;
        let repo_name = &self.event.repository.name;
        let pr_number = self.event.pull_request.number;

        let repo_model = GetOrCreateRepositoryUseCase {
            db_service: self.db_service,
            config: self.config,
            repo_name,
            repo_owner,
        }
        .run()
        .await?;

        match self
            .db_service
            .pull_requests_get(repo_owner, repo_name, pr_number)
            .await?
        {
            Some(_p) => Ok(PullRequestOpenedStatus::AlreadyCreated),
            None => {
                if Self::should_create_pull_request(self.config, &repo_model, &self.event) {
                    let pr_model = self
                        .db_service
                        .pull_requests_create(
                            PullRequest {
                                number: self.event.pull_request.number,
                                ..Default::default()
                            }
                            .with_repository(&repo_model),
                        )
                        .await?;

                    // Get upstream pull request
                    let upstream_pr = self
                        .api_service
                        .pulls_get(&repo_model.owner, &repo_model.name, pr_model.number)
                        .await?;

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

                    if self.config.server_enable_welcome_comments {
                        PostWelcomeCommentUseCase {
                            api_service: self.api_service,
                            repo_owner,
                            repo_name,
                            pr_number,
                            pr_author: &self.event.pull_request.user.login,
                        }
                        .run()
                        .await?;
                    }

                    // Now, handle commands from body.
                    let commands = CommandParser::parse_commands(
                        self.config,
                        self.event.pull_request.body.as_deref().unwrap_or_default(),
                    );

                    let mut ctx = CommandContext {
                        config: self.config,
                        api_service: self.api_service,
                        db_service: self.db_service,
                        lock_service: self.lock_service,
                        repo_owner,
                        repo_name,
                        pr_number,
                        upstream_pr: &upstream_pr,
                        comment_id: 0,
                        comment_author: &self.event.pull_request.user.login,
                    };

                    CommandExecutor::execute_commands(&mut ctx, commands).await?;

                    Ok(PullRequestOpenedStatus::Created)
                } else {
                    Ok(PullRequestOpenedStatus::Ignored)
                }
            }
        }
    }

    pub fn should_create_pull_request(
        config: &Config,
        repo_model: &Repository,
        event: &GhPullRequestEvent,
    ) -> bool {
        if repo_model.manual_interaction {
            if let Some(body) = &event.pull_request.body {
                // Check for magic instruction to enable bot
                let commands = CommandParser::parse_commands(config, body);
                for command in commands.into_iter().flatten() {
                    if let Command::Admin(AdminCommand::Enable) = command {
                        return true;
                    }
                }
            }

            false
        } else {
            true
        }
    }
}
