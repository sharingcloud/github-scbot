//! Welcome module.

use github_scbot_api::{adapter::IAPIAdapter, comments::post_comment};
use github_scbot_database::models::{PullRequestModel, RepositoryModel};

use crate::errors::Result;

/// Post welcome comment on a pull request.
pub async fn post_welcome_comment(
    api_adapter: &impl IAPIAdapter,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    post_comment(
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
