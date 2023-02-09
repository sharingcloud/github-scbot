use crate::Result;
use github_scbot_ghapi_interface::{comments::CommentApi, ApiService};

pub struct PostWelcomeCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub pr_author: &'a str,
}

impl<'a> PostWelcomeCommentUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        CommentApi::post_comment(
            self.api_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                self.pr_author
            ),
        )
        .await?;

        Ok(())
    }
}
