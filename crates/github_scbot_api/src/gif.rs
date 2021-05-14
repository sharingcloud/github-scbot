//! GIF module.

use std::collections::HashMap;

use github_scbot_conf::Config;
use rand::prelude::*;
use serde::Deserialize;

use crate::Result;

const GIF_API_URL: &str = "https://g.tenor.com/v1";
const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

#[allow(non_camel_case_types)]
#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum GifFormat {
    Gif,
    MediumGif,
    TinyGif,
    NanoGif,
    Mp4,
    LoopedMp4,
    TinyMp4,
    NanoMp4,
    WebM,
    TinyWebM,
    NanoWebM,
    WebP_Transparent,
}

#[derive(Deserialize)]
struct MediaObject {
    url: String,
    size: usize,
}

#[derive(Deserialize)]
struct GifObject {
    media: Vec<HashMap<GifFormat, MediaObject>>,
}

#[derive(Deserialize)]
struct RandomResponse {
    results: Vec<GifObject>,
}

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

/// Get random GIF for query.
pub async fn random_gif_for_query(config: &Config, search: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut response: RandomResponse = client
        .get(&format!("{}/random", GIF_API_URL))
        .query(&[
            ("q", search),
            ("key", &config.tenor_api_key),
            ("limit", "3"),
            ("locale", "en_US"),
            ("contentfilter", "low"),
            ("media_filter", "basic"),
            ("ar_range", "all"),
        ])
        .send()
        .await?
        .json()
        .await?;

    if response.results.is_empty() {
        Ok(String::new())
    } else {
        let mut url = String::new();

        // Shuffle responses
        let mut rng = thread_rng();
        response.results.shuffle(&mut rng);

        // Get first media found
        for result in &response.results {
            for media in &result.media {
                for key in GIF_KEYS {
                    if media.contains_key(key) && media[key].size < MAX_GIF_SIZE_BYTES {
                        url = media[key].url.clone();
                        break;
                    }
                }
            }
        }

        Ok(url)
    }
}
