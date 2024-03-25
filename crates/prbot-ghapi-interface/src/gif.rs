use std::collections::HashMap;

use serde::Deserialize;

/// Gif format.
#[allow(non_camel_case_types)]
#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GifFormat {
    /// Standard GIF.
    Gif,
    /// Medium GIF.
    MediumGif,
    /// Tiny GIF.
    TinyGif,
    /// Nano GIF.
    NanoGif,
    /// MP4.
    Mp4,
    /// Looped MP4.
    LoopedMp4,
    /// Tiny MP4.
    TinyMp4,
    /// Nano MP4.
    NanoMp4,
    /// WebM.
    WebM,
    /// Tiny WebM.
    TinyWebM,
    /// Nano WebM.
    NanoWebM,
    /// Transparent WebP.
    WebP_Transparent,
}

/// Media object.
#[derive(Deserialize, Clone, Debug)]
pub struct MediaObject {
    /// Media URL.
    pub url: String,
    /// Media size.
    pub size: Option<usize>,
}

/// Gif object.
#[derive(Deserialize, Clone, Default, Debug)]
pub struct GifObject {
    /// Media dict.
    pub media: Vec<HashMap<GifFormat, MediaObject>>,
}

/// Gif response.
#[derive(Deserialize, Clone, Default, Debug)]
pub struct GifResponse {
    /// Results.
    pub results: Vec<GifObject>,
}
