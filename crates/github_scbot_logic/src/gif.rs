//! GIF logic module.

use github_scbot_api::{comments::post_comment, gif::random_gif_for_query};
use github_scbot_conf::Config;
use github_scbot_database::models::{PullRequestModel, RepositoryModel};

use crate::Result;

/// Post random GIF comment.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `search_terms` - Search terms
pub async fn post_random_gif_comment(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    search_terms: &str,
) -> Result<bool> {
    let random_gif = random_gif_for_query(config, search_terms).await?;

    if random_gif.is_empty() {
        Ok(false)
    } else {
        let body = format!(
            "![GIF]({url})\n[_Via Tenor_](https://tenor.com/)",
            url = random_gif
        );

        post_comment(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &body,
        )
        .await?;

        Ok(true)
    }
}
