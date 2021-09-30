//! Gif logic module.

use github_scbot_conf::Config;
use github_scbot_database::models::{PullRequestModel, RepositoryModel};
use github_scbot_ghapi::{
    adapter::IAPIAdapter, comments::post_comment, gif::random_gif_from_query,
};

use crate::Result;

/// Gif poster.
pub struct GifPoster;

impl GifPoster {
    /// Post random GIF comment.
    pub async fn post_random_gif_comment(
        config: &Config,
        api_adapter: &dyn IAPIAdapter,
        repo_model: &RepositoryModel,
        pr_model: &PullRequestModel,
        search_terms: &str,
    ) -> Result<()> {
        let body = Self::generate_random_gif_comment(config, api_adapter, search_terms).await?;
        post_comment(
            api_adapter,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &body,
        )
        .await?;

        Ok(())
    }

    /// Generate random GIF comment.
    pub async fn generate_random_gif_comment(
        config: &Config,
        api_adapter: &dyn IAPIAdapter,
        search_terms: &str,
    ) -> Result<String> {
        let random_gif = random_gif_from_query(config, api_adapter, search_terms).await?;

        match random_gif {
            None => Ok(format!(
                "No compatible GIF found for query `{}` :cry:",
                search_terms
            )),
            Some(url) => Ok(format!(
                "![GIF]({url})\n[_Via Tenor_](https://tenor.com/)",
                url = url
            )),
        }
    }
}
