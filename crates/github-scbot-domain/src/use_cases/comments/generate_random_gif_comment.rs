use github_scbot_core::config::Config;
use github_scbot_ghapi_interface::{gif_api::GifApi, ApiService};

use crate::Result;

pub struct GenerateRandomGifCommentUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub search_terms: &'a str,
}

impl<'a> GenerateRandomGifCommentUseCase<'a> {
    pub async fn run(&mut self) -> Result<String> {
        let random_gif =
            GifApi::random_gif_from_query(self.config, self.api_service, self.search_terms).await?;

        match random_gif {
            None => Ok(format!(
                "No compatible GIF found for query `{}` :cry:",
                self.search_terms
            )),
            Some(url) => Ok(format!(
                "![GIF]({url})\n[_Via Tenor_](https://tenor.com/)",
                url = url
            )),
        }
    }
}
