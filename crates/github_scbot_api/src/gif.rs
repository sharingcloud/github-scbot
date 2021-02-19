//! GIF module.

use std::collections::HashMap;

use serde::Deserialize;

use crate::Result;
use github_scbot_conf::Config;

const GIF_API_URL: &str = "https://g.tenor.com/v1";

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
}

#[derive(Deserialize)]
struct MediaObject {
    url: String,
}

#[derive(Deserialize)]
struct GifObject {
    media: Vec<HashMap<GifFormat, MediaObject>>,
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
            ("limit", "10"),
            ("contentfilter", "low"),
            ("media_filter", "minimal"),
            ("ar_range", "wide"),
        ])
        .send()
        .await?
        .json()
        .await?;

    if response.results.is_empty() {
        Ok(String::new())
    } else {
        let mut url = String::new();

        // Get first media found
        'top: for result in &response.results {
            for media in &result.media {
                if media.contains_key(&GifFormat::Gif) {
                    url = media[&GifFormat::Gif].url.clone();
                    break 'top;
                }
            }
        }

        Ok(url)
    }
}
