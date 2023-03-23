use async_trait::async_trait;

use crate::{use_cases::gifs::RandomGifFromQueryUseCaseInterface, Result};

#[mockall::automock]
#[async_trait(?Send)]
pub trait GenerateRandomGifCommentUseCaseInterface {
    async fn run(&self, search_terms: &str) -> Result<String>;
}

pub struct GenerateRandomGifCommentUseCase<'a> {
    pub random_gif_from_query: &'a dyn RandomGifFromQueryUseCaseInterface,
}

#[async_trait(?Send)]
impl<'a> GenerateRandomGifCommentUseCaseInterface for GenerateRandomGifCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(search_terms), ret)]
    async fn run(&self, search_terms: &str) -> Result<String> {
        let random_gif = self.random_gif_from_query.run(search_terms).await?;

        match random_gif {
            None => Ok(format!(
                "No compatible GIF found for query `{}` :cry:",
                search_terms
            )),
            Some(url) => Ok(format!(
                "![GIF]({url})\n[_Via Tenor_](https://tenor.com/)",
                url = url
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::gifs::MockRandomGifFromQueryUseCaseInterface;

    #[tokio::test]
    async fn run_not_found() {
        let mut random_gif_from_query = MockRandomGifFromQueryUseCaseInterface::new();
        random_gif_from_query
            .expect_run()
            .once()
            .withf(|search| search == "random")
            .return_once(|_| Ok(None));

        let response = GenerateRandomGifCommentUseCase {
            random_gif_from_query: &random_gif_from_query,
        }
        .run("random")
        .await
        .unwrap();

        assert!(response.starts_with("No compatible GIF found"));
    }

    #[tokio::test]
    async fn run_found() {
        let mut random_gif_from_query = MockRandomGifFromQueryUseCaseInterface::new();
        random_gif_from_query
            .expect_run()
            .once()
            .withf(|search| search == "random")
            .return_once(|_| Ok(Some("http://local".into())));

        let response = GenerateRandomGifCommentUseCase {
            random_gif_from_query: &random_gif_from_query,
        }
        .run("random")
        .await
        .unwrap();

        assert!(response.starts_with("![GIF]("))
    }
}
