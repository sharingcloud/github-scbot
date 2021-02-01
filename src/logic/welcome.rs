//! Welcome module.

use crate::{
    api::comments::post_comment,
    database::models::{PullRequestModel, RepositoryModel},
    logic::errors::Result,
    webhook::constants::ENV_DISABLE_WELCOME_COMMENTS,
};

/// Post welcome comment on a pull request.
///
/// # Arguments
///
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `pr_author` - Pull request author
pub async fn post_welcome_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    if std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        post_comment(
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
