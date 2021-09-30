//! GIF module.

use github_scbot_conf::Config;
use rand::prelude::*;

use crate::{
    adapter::{GifFormat, GifResponse, IAPIAdapter},
    Result,
};

const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

/// Get random GIF from query.
pub async fn random_gif_from_query(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    search: &str,
) -> Result<Option<String>> {
    Ok(random_gif_from_response(
        api_adapter
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
    get_first_matching_gif(&response)
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use super::*;
    use crate::adapter::{GifObject, MediaObject};

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
        assert_eq!(get_first_matching_gif(&response), Some("http://aaa".into()));

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
        assert_eq!(get_first_matching_gif(&response), None);
    }
}
