use async_trait::async_trait;
use prbot_ghapi_interface::gif::{GifFormat, GifResponse};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

const MAX_GIF_SIZE_BYTES: usize = 2_000_000;

const GIF_KEYS: &[GifFormat] = &[
    GifFormat::Gif,
    GifFormat::MediumGif,
    GifFormat::TinyGif,
    GifFormat::NanoGif,
];

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait RandomGifFromQueryInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, search: &str) -> Result<Option<String>>;
}

#[derive(Component)]
#[shaku(interface = RandomGifFromQueryInterface)]
pub(crate) struct RandomGifFromQuery;

#[async_trait]
impl RandomGifFromQueryInterface for RandomGifFromQuery {
    #[tracing::instrument(skip(self, ctx), fields(search), ret)]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, search: &str) -> Result<Option<String>> {
        Ok(self.random_gif_from_response(
            ctx.api_service
                .gif_search(&ctx.config.tenor_api_key, search)
                .await?,
            ctx.config.random_seed,
        ))
    }
}

impl RandomGifFromQuery {
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

    fn random_gif_from_response(
        &self,
        mut response: GifResponse,
        rand_seed: u64,
    ) -> Option<String> {
        let mut rng = ChaCha8Rng::seed_from_u64(rand_seed);

        response.results.shuffle(&mut rng);
        self.get_first_matching_gif(&response)
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;
    use prbot_ghapi_interface::{
        gif::{GifObject, MediaObject},
        MockApiService,
    };

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn run() {
        let mut ctx = CoreContextTest::new();
        ctx.config.tenor_api_key = "gifkey".into();
        ctx.config.random_seed = 1;

        ctx.api_service = {
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

        let url = RandomGifFromQuery
            .run(&ctx.as_context(), "random")
            .await
            .unwrap();
        assert_eq!(url, Some("http://aaa".into()));
    }

    #[test]
    fn get_first_matching_gif() {
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
            RandomGifFromQuery.get_first_matching_gif(&response),
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
        assert_eq!(RandomGifFromQuery.get_first_matching_gif(&response), None);
    }
}
