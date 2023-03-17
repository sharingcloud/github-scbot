//! GIF module.

use github_scbot_core::config::Config;
use rand::prelude::*;

use crate::gif::{GifFormat, GifResponse};
use crate::{ApiService, Result};

const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

/// Gif API.
pub struct GifApi;

impl GifApi {
    /// Get random GIF from query.
    pub async fn random_gif_from_query(
        config: &Config,
        api_service: &dyn ApiService,
        search: &str,
    ) -> Result<Option<String>> {
        Ok(Self::random_gif_from_response(
            api_service
                .gif_search(&config.tenor_api_key, search)
                .await?,
        ))
    }

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
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use super::*;
    use crate::gif::{GifObject, MediaObject};

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
            GifApi::get_first_matching_gif(&response),
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
        assert_eq!(GifApi::get_first_matching_gif(&response), None);
    }
}
