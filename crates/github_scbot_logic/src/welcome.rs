//! Welcome module.

use github_scbot_api::comments::post_comment;
use github_scbot_core::Config;
use github_scbot_database::models::{PullRequestModel, RepositoryModel};

use crate::errors::Result;

/// Post welcome comment on a pull request.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `pr_author` - Pull request author
pub async fn post_welcome_comment(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    if config.server_enable_welcome_comments {
        post_comment(
            config,
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
    }

    Ok(())
}
