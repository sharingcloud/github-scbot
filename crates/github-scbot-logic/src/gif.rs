//! Gif logic module.

use github_scbot_core::config::Config;
use github_scbot_ghapi::{adapter::ApiService, comments::CommentApi, gif::GifApi};

use crate::Result;

/// Gif poster.
pub struct GifPoster;

impl GifPoster {
    /// Post random GIF comment.
    pub async fn post_random_gif_comment(
        config: &Config,
        api_adapter: &dyn ApiService,
        owner: &str,
        name: &str,
        number: u64,
        search_terms: &str,
    ) -> Result<()> {
        let body = Self::generate_random_gif_comment(config, api_adapter, search_terms).await?;
        CommentApi::post_comment(api_adapter, owner, name, number, &body).await?;

        Ok(())
    }

    /// Generate random GIF comment.
    pub async fn generate_random_gif_comment(
        config: &Config,
        api_adapter: &dyn ApiService,
        search_terms: &str,
    ) -> Result<String> {
        let random_gif = GifApi::random_gif_from_query(config, api_adapter, search_terms).await?;

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
