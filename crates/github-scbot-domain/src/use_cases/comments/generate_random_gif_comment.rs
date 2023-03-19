use github_scbot_config::Config;
use github_scbot_ghapi_interface::ApiService;

use crate::{use_cases::gifs::RandomGifFromQueryUseCase, Result};

pub struct GenerateRandomGifCommentUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub search_terms: &'a str,
}

impl<'a> GenerateRandomGifCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.search_terms), ret)]
    pub async fn run(&mut self) -> Result<String> {
        let random_gif = RandomGifFromQueryUseCase {
            config: self.config,
            api_service: self.api_service,
            search: self.search_terms,
        }
        .run()
        .await?;

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
