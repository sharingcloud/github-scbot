//! Welcome

use crate::api::comments::post_comment_for_repo;
use crate::database::models::{PullRequestModel, RepositoryModel};
use crate::webhook::constants::ENV_DISABLE_WELCOME_COMMENTS;
use crate::webhook::errors::Result;

#[allow(clippy::cast_sign_loss)]
pub async fn post_welcome_comment(
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    pr_author: &str,
) -> Result<()> {
    if std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        post_comment_for_repo(
            repo_model,
            pr_model.number as u64,
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
