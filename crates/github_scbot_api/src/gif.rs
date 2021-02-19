//! GIF module.

use std::collections::HashMap;

use serde::Deserialize;

use crate::Result;
use github_scbot_conf::Config;

const GIF_API_URL: &str = "https://g.tenor.com/v1";

#[derive(Deserialize, PartialEq, Eq, Hash)]
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
}

#[derive(Deserialize)]
struct MediaObject {
    url: String,
}

#[derive(Deserialize)]
struct GifObject {
    media: HashMap<GifFormat, MediaObject>,
}

#[derive(Deserialize)]
struct RandomResponse {
    results: Vec<GifObject>,
}

/// Get random GIF for query.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `search` - Search string
pub async fn random_gif_for_query(config: &Config, search: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response: RandomResponse = client
        .get(&format!("{}/random", GIF_API_URL))
        .query(&[
            ("q", search),
            ("key", &config.tenor_api_key),
            ("limit", "1"),
            ("contentfilter", "low"),
            ("media_filter", "minimal"),
            ("ar_range", "all"),
        ])
        .send()
        .await?
        .json()
        .await?;

    // Get first media found
    if response.results.is_empty() {
        Ok(String::new())
    } else {
        let key = response.results[0].media.keys().next().unwrap();
        Ok(response.results[0].media[key].url.clone())
    }
}
