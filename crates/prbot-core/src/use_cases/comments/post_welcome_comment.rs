use async_trait::async_trait;
use prbot_ghapi_interface::comments::CommentApi;
use prbot_models::PullRequestHandle;
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait PostWelcomeCommentInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        pr_author: &str,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = PostWelcomeCommentInterface)]
pub(crate) struct PostWelcomeComment;

#[async_trait]
impl PostWelcomeCommentInterface for PostWelcomeComment {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle, pr_author))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        pr_author: &str,
    ) -> Result<()> {
        CommentApi::post_comment(
            ctx.config,
            ctx.api_service,
            pr_handle.repository_path().owner(),
            pr_handle.repository_path().name(),
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
    use prbot_ghapi_interface::MockApiService;

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();
        ctx.api_service = {
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

        PostWelcomeComment
            .run(&ctx.as_context(), &("me", "test", 1).into(), "foo")
            .await
            .unwrap();
    }
}
