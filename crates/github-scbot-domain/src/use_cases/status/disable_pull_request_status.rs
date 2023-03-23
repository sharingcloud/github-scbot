use async_trait::async_trait;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{types::GhCommitStatus, ApiService};

use super::utils::VALIDATION_STATUS_MESSAGE;
use crate::{use_cases::summary::DeleteSummaryCommentUseCaseInterface, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait DisablePullRequestStatusUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()>;
}

pub struct DisablePullRequestStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub delete_summary_comment: &'a dyn DeleteSummaryCommentUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> DisablePullRequestStatusUseCaseInterface for DisablePullRequestStatusUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        let sha = self
            .api_service
            .pulls_get(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?
            .head
            .sha;

        self.api_service
            .commit_statuses_update(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                &sha,
                GhCommitStatus::Success,
                VALIDATION_STATUS_MESSAGE,
                "Bot disabled.",
            )
            .await?;

        self.delete_summary_comment.run(pr_handle).await
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::{
        types::{GhBranch, GhPullRequest},
        MockApiService,
    };

    use super::*;
    use crate::use_cases::summary::MockDeleteSummaryCommentUseCaseInterface;

    #[tokio::test]
    async fn run() {
        let api_service = {
            let mut api_service = MockApiService::new();
            api_service
                .expect_pulls_get()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 1,
                        head: GhBranch {
                            sha: "abcdef".into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                });

            api_service
                .expect_commit_statuses_update()
                .once()
                .withf(|owner, name, sha, status, title, body| {
                    owner == "me"
                        && name == "test"
                        && sha == "abcdef"
                        && *status == GhCommitStatus::Success
                        && title == VALIDATION_STATUS_MESSAGE
                        && body == "Bot disabled."
                })
                .return_once(|_, _, _, _, _, _| Ok(()));

            api_service
        };

        let delete_summary_comment = {
            let mut delete_summary_comment = MockDeleteSummaryCommentUseCaseInterface::new();
            delete_summary_comment
                .expect_run()
                .once()
                .withf(|pr_handle| pr_handle == &("me", "test", 1).into())
                .return_once(|_| Ok(()));
            delete_summary_comment
        };

        DisablePullRequestStatusUseCase {
            api_service: &api_service,
            delete_summary_comment: &delete_summary_comment,
        }
        .run(&("me", "test", 1).into())
        .await
        .unwrap()
    }
}
