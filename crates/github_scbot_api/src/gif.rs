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
    api_adapter: &impl IAPIAdapter,
    search: &str,
) -> Result<Option<String>> {
    Ok(random_gif_from_response(
        api_adapter
            .gif_search(&config.tenor_api_key, search)
            .await?,
    ))
}

fn random_gif_from_response(mut response: GifResponse) -> Option<String> {
    if response.results.is_empty() {
        None
    } else {
        let mut url = String::new();

        // Shuffle responses
        let mut rng = thread_rng();
        response.results.shuffle(&mut rng);

        // Get first media found
        for result in &response.results {
            for media in &result.media {
                for key in GIF_KEYS {
                    if media.contains_key(key) {
                        if let Some(size) = media[key].size {
                            if size < MAX_GIF_SIZE_BYTES {
                                url = media[key].url.clone();
                                break;
                            }
                        }
                    }
                }
            }
        }

        Some(url)
    }
}
