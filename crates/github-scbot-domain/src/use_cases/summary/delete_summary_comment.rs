use async_trait::async_trait;
use github_scbot_database_interface::DbService;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;

use super::utils::sender::SummaryCommentSender;
use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait DeleteSummaryCommentUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()>;
}

pub struct DeleteSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a dyn DbService,
}

#[async_trait(?Send)]
impl<'a> DeleteSummaryCommentUseCaseInterface for DeleteSummaryCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    async fn run(&self, pr_handle: &PullRequestHandle) -> Result<()> {
        SummaryCommentSender::delete(self.api_service, self.db_service, pr_handle)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{PullRequest, Repository};
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[tokio::test]
    async fn run_no_existing_id() {
        let api_service = MockApiService::new();
        let db_service = MemoryDb::new();

        DeleteSummaryCommentUseCase {
            api_service: &api_service,
            db_service: &db_service,
        }
        .run(&("me", "test", 1).into())
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn run_existing_id() {
        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_delete()
                .once()
                .withf(|owner, name, comment_id| {
                    owner == "me" && name == "test" && comment_id == &1
                })
                .return_once(|_, _, _| Ok(()));

            svc
        };

        let db_service = {
            let svc = MemoryDb::new();
            let repo = svc
                .repositories_create(Repository {
                    owner: "me".into(),
                    name: "test".into(),
                    ..Default::default()
                })
                .await
                .unwrap();

            svc.pull_requests_create(
                PullRequest {
                    number: 1,
                    status_comment_id: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();

            svc
        };

        DeleteSummaryCommentUseCase {
            api_service: &api_service,
            db_service: &db_service,
        }
        .run(&("me", "test", 1).into())
        .await
        .unwrap();
    }
}
