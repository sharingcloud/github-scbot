//! Welcome module.

use github_scbot_database::models::{PullRequestModel, RepositoryModel};
use github_scbot_ghapi::{adapter::IAPIAdapter, comments::CommentApi};

use crate::errors::Result;

/// Post welcome comment on a pull request.
pub async fn post_welcome_comment(
    api_adapter: &dyn IAPIAdapter,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    CommentApi::post_comment(
        api_adapter,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &format!(
            ":tada: Welcome, _{}_ ! :tada:\n\
        Thanks for your pull request, it will be reviewed soon. :clock2:",
            pr_author
        ),
    )
    .await?;

    Ok(())
}
