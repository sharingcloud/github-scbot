use async_trait::async_trait;
use github_scbot_config::Config;
use github_scbot_ghapi_interface::{
    gif::{GifFormat, GifResponse},
    ApiService,
};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::Result;

const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait RandomGifFromQueryUseCaseInterface {
    async fn run(&self, search: &str) -> Result<Option<String>>;
}

pub struct RandomGifFromQueryUseCase<'a> {
    pub config: &'a Config,
    pub api_service: &'a dyn ApiService,
    pub rand_seed: u64,
}

#[async_trait(?Send)]
impl<'a> RandomGifFromQueryUseCaseInterface for RandomGifFromQueryUseCase<'a> {
    #[tracing::instrument(skip(self), fields(search), ret)]
    async fn run(&self, search: &str) -> Result<Option<String>> {
        Ok(self.random_gif_from_response(
            self.api_service
                .gif_search(&self.config.tenor_api_key, search)
                .await?,
        ))
    }
}

impl<'a> RandomGifFromQueryUseCase<'a> {
    fn get_first_matching_gif(&self, response: &GifResponse) -> Option<String> {
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

    fn random_gif_from_response(&self, mut response: GifResponse) -> Option<String> {
        let mut rng = ChaCha8Rng::seed_from_u64(self.rand_seed);

        response.results.shuffle(&mut rng);
        self.get_first_matching_gif(&response)
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::{
        gif::{GifObject, MediaObject},
        MockApiService,
    };
    use maplit::hashmap;

    use super::*;

    #[tokio::test]
    async fn run() {
        let mut config = Config::from_env();
        config.tenor_api_key = "gifkey".into();

        let api_service = {
            let mut svc = MockApiService::new();
            svc.expect_gif_search()
                .once()
                .withf(|key, search| key == "gifkey" && search == "random")
                .return_once(|_, _| {
                    Ok(GifResponse {
                        results: vec![
                            GifObject {
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
                            },
                            GifObject {
                                media: vec![hashmap! {
                                    GifFormat::Mp4 => MediaObject {
                                        url: "http://local.test/2".into(),
                                        size: None
                                    },
                                    GifFormat::NanoGif => MediaObject {
                                        url: "http://bbb".into(),
                                        size: Some(12)
                                    }
                                }],
                            },
                        ],
                    })
                });
            svc
        };

        let uc = RandomGifFromQueryUseCase {
            api_service: &api_service,
            config: &config,
            rand_seed: 1,
        };

        let url = uc.run("random").await.unwrap();
        assert_eq!(url, Some("http://aaa".into()));
    }

    #[test]
    fn get_first_matching_gif() {
        let config = Config::from_env();
        let api_service = MockApiService::new();

        let uc = RandomGifFromQueryUseCase {
            api_service: &api_service,
            config: &config,
            rand_seed: 1,
        };

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
            uc.get_first_matching_gif(&response),
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
        assert_eq!(uc.get_first_matching_gif(&response), None);
    }
}
