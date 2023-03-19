use github_scbot_config::Config;
use github_scbot_ghapi_interface::{
    gif::{GifFormat, GifResponse},
    ApiService,
};
use rand::prelude::*;

use crate::Result;

const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

pub struct RandomGifFromQueryUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub search: &'a str,
}

impl<'a> RandomGifFromQueryUseCase<'a> {
    fn get_first_matching_gif(response: &GifResponse) -> Option<String> {
        if !response.results.is_empty() {
            // Get first media found
            for result in &response.results {
                for media in &result.media {
                    for key in GIF_KEYS {
                        if media.contains_key(key) {
                            if let Some(size) = media[key].size {
                                if size < MAX_GIF_SIZE_BYTES {
                                    return Some(media[key].url.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn random_gif_from_response(mut response: GifResponse) -> Option<String> {
        response.results.shuffle(&mut thread_rng());
        Self::get_first_matching_gif(&response)
    }

    #[tracing::instrument(skip(self), fields(self.search), ret)]
    pub async fn run(&mut self) -> Result<Option<String>> {
        Ok(Self::random_gif_from_response(
            self.api_service
                .gif_search(&self.config.tenor_api_key, self.search)
                .await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::gif::{GifObject, MediaObject};
    use maplit::hashmap;

    use super::*;

    #[test]
    fn test_get_first_matching_gif() {
        let response = GifResponse {
            results: vec![GifObject {
                media: vec![hashmap! {
                    GifFormat::Mp4 => MediaObject {
                        url: "http://local.test".into(),
                        size: None
                    },
                    GifFormat::NanoGif => MediaObject {
                        url: "http://aaa".into(),
                        size: Some(12)
                    }
                }],
            }],
        };
        assert_eq!(
            RandomGifFromQueryUseCase::get_first_matching_gif(&response),
            Some("http://aaa".into())
        );

        let response = GifResponse {
            results: vec![GifObject {
                media: vec![hashmap! {
                    GifFormat::Mp4 => MediaObject {
                        url: "http://local.test".into(),
                        size: None
                    },
                    GifFormat::Gif => MediaObject {
                        url: "http://aaa".into(),
                        size: Some(MAX_GIF_SIZE_BYTES)
                    }
                }],
            }],
        };
        assert_eq!(
            RandomGifFromQueryUseCase::get_first_matching_gif(&response),
            None
        );
    }
}
