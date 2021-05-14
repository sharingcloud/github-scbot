//! GIF logic module.

use github_scbot_api::{comments::post_comment, gif::random_gif_for_query};
use github_scbot_conf::Config;
use github_scbot_database::models::{PullRequestModel, RepositoryModel};

use crate::Result;

/// Post random GIF comment.
pub async fn post_random_gif_comment(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    search_terms: &str,
) -> Result<()> {
    if let Some(body) = generate_random_gif_comment(config, search_terms).await? {
        post_comment(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &body,
        )
        .await?;
    }

    Ok(())
}

/// Generate random GIF comment.
pub async fn generate_random_gif_comment(
    config: &Config,
    search_terms: &str,
) -> Result<Option<String>> {
    let random_gif = random_gif_for_query(config, search_terms).await?;

    if random_gif.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "![GIF]({url})\n[_Via Tenor_](https://tenor.com/)",
            url = random_gif
        )))
    }
}
