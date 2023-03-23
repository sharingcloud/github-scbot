use async_trait::async_trait;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{comments::CommentApi, ApiService};

use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait PostWelcomeCommentUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, pr_author: &str) -> Result<()>;
}

pub struct PostWelcomeCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

#[async_trait(?Send)]
impl<'a> PostWelcomeCommentUseCaseInterface for PostWelcomeCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle, pr_author))]
    async fn run(&self, pr_handle: &PullRequestHandle, pr_author: &str) -> Result<()> {
        CommentApi::post_comment(
            self.api_service,
            pr_handle.repository().owner(),
            pr_handle.repository().name(),
            pr_handle.number(),
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                pr_author
            ),
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[tokio::test]
    async fn run() {
        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_comments_post()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me"
                        && name == "test"
                        && number == &1
                        && body.starts_with(":tada: Welcome, _foo_ !")
                })
                .return_once(|_, _, _, _| Ok(1));

            svc
        };

        PostWelcomeCommentUseCase {
            api_service: &api_service,
        }
        .run(&("me", "test", 1).into(), "foo")
        .await
        .unwrap();
    }
}
