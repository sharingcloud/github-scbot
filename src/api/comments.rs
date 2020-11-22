//! Comments API module

use std::convert::TryInto;

use eyre::Result;

use super::constants::ENV_DISABLE_WELCOME_COMMENTS;
use super::get_client;

pub async fn post_welcome_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: i32,
    pr_author: &str,
) -> Result<()> {
    if !cfg!(test) && std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        let client = get_client().await?;

        client
            .issues(repo_owner, repo_name)
            .create_comment(
                pr_number.try_into()?,
                format!(
                    ":tada: Welcome, _{}_ ! :tada:\n\
                    Thanks for your pull request, it will be reviewed soon. :clock2:",
                    pr_author
                ),
            )
            .await?;
    }

    Ok(())
}
