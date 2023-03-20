use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::{comments::CommentApi, ApiService};

use crate::Result;

pub struct PostWelcomeCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

impl<'a> PostWelcomeCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle, pr_author))]
    pub async fn run(&self, pr_handle: &PullRequestHandle, pr_author: &str) -> Result<()> {
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
