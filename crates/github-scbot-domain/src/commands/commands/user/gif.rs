use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use github_scbot_ghapi_interface::types::GhReactionType;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    use_cases::{
        comments::{GenerateRandomGifCommentUseCase, GenerateRandomGifCommentUseCaseInterface},
        gifs::RandomGifFromQueryUseCase,
    },
    Result,
};

pub struct GifCommand {
    search_terms: String,
}

impl GifCommand {
    pub fn new(search_terms: String) -> Self {
        Self { search_terms }
    }
}

#[async_trait(?Send)]
impl BotCommand for GifCommand {
    async fn handle(&self, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::builder()
            .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
            .with_action(ResultAction::PostComment(
                GenerateRandomGifCommentUseCase {
                    random_gif_from_query: &RandomGifFromQueryUseCase {
                        config: ctx.config,
                        api_service: ctx.api_service,
                        // Use the current timestamp in milliseconds as a seed, this is not used for cryptography,
                        // so it should be more than enough.
                        rand_seed: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    },
                }
                .run(&self.search_terms)
                .await?,
            ))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::gif::{GifFormat, GifObject, GifResponse, MediaObject};
    use maplit::hashmap;

    use super::*;
    use crate::commands::CommandContextTest;

    #[tokio::test]
    async fn test_valid() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.config.tenor_api_key = "gifkey".into();

        ctx.api_service
            .expect_gif_search()
            .once()
            .withf(|key, search| key == "gifkey" && search == "what")
            .returning(|_, _| {
                Ok(GifResponse {
                    results: vec![GifObject {
                        media: vec![hashmap!(
                            GifFormat::Gif => MediaObject {
                                url: "http://url".into(),
                                size: Some(123)
                            }
                        )],
                    }],
                })
            });

        let result = GifCommand::new("what".into())
            .handle(&ctx.as_context())
            .await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment(
                    "![GIF](http://url)\n[_Via Tenor_](https://tenor.com/)".into()
                )
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid() -> Result<()> {
        let mut ctx = CommandContextTest::new();
        ctx.config.tenor_api_key = "gifkey".into();

        ctx.api_service
            .expect_gif_search()
            .once()
            .withf(|key, search| key == "gifkey" && search == "what")
            .returning(|_, _| Ok(GifResponse { results: vec![] }));

        let result = GifCommand::new("what".into())
            .handle(&ctx.as_context())
            .await?;
        assert!(!result.should_update_status);
        assert_eq!(
            result.result_actions,
            vec![
                ResultAction::AddReaction(GhReactionType::Eyes),
                ResultAction::PostComment("No compatible GIF found for query `what` :cry:".into())
            ]
        );

        Ok(())
    }
}
