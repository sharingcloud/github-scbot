use async_trait::async_trait;
use shaku::{Component, HasComponent, Interface};

use crate::{use_cases::gifs::RandomGifFromQueryInterface, CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait GenerateRandomGifCommentInterface: Interface {
    async fn run<'a>(&self, ctx: &CoreContext<'a>, search_terms: &str) -> Result<String>;
}

#[derive(Component)]
#[shaku(interface = GenerateRandomGifCommentInterface)]
pub(crate) struct GenerateRandomGifComment;

#[async_trait]
impl GenerateRandomGifCommentInterface for GenerateRandomGifComment {
    #[tracing::instrument(skip(self, ctx), fields(search_terms), ret)]
    async fn run<'a>(&self, ctx: &CoreContext<'a>, search_terms: &str) -> Result<String> {
        let random_gif_from_query: &dyn RandomGifFromQueryInterface = ctx.core_module.resolve_ref();
        let random_gif = random_gif_from_query.run(ctx, search_terms).await?;

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
    use crate::{
        context::tests::CoreContextTest,
        use_cases::{
            comments::{
                generate_random_gif_comment::GenerateRandomGifComment,
                GenerateRandomGifCommentInterface,
            },
            gifs::{MockRandomGifFromQueryInterface, RandomGifFromQueryInterface},
        },
        CoreModule,
    };

    #[tokio::test]
    async fn run_not_found() {
        let mut ctx = CoreContextTest::new();

        let mut random_gif_from_query = MockRandomGifFromQueryInterface::new();
        random_gif_from_query
            .expect_run()
            .once()
            .withf(|_, search| search == "random")
            .return_once(|_, _| Ok(None));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn RandomGifFromQueryInterface>(Box::new(
                random_gif_from_query,
            ))
            .build();

        let response = GenerateRandomGifComment
            .run(&ctx.as_context(), "random")
            .await
            .unwrap();

        assert!(response.starts_with("No compatible GIF found"));
    }

    #[tokio::test]
    async fn run_found() {
        let mut ctx = CoreContextTest::new();

        let mut random_gif_from_query = MockRandomGifFromQueryInterface::new();
        random_gif_from_query
            .expect_run()
            .once()
            .withf(|_, search| search == "random")
            .return_once(|_, _| Ok(Some("http://local".into())));

        ctx.core_module = CoreModule::builder()
            .with_component_override::<dyn RandomGifFromQueryInterface>(Box::new(
                random_gif_from_query,
            ))
            .build();

        let response = GenerateRandomGifComment
            .run(&ctx.as_context(), "random")
            .await
            .unwrap();

        assert!(response.starts_with("![GIF]("))
    }
}
