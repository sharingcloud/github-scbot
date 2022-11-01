use github_scbot_core::types::issues::GhReactionType;

use async_trait::async_trait;

use crate::{
    commands::{
        command::{CommandExecutionResult, ResultAction},
        BotCommand, CommandContext,
    },
    gif::GifPoster,
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
                GifPoster::generate_random_gif_comment(
                    ctx.config,
                    ctx.api_adapter,
                    &self.search_terms,
                )
                .await?,
            ))
            .build())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi::adapter::GifFormat;
    use github_scbot_ghapi::adapter::GifObject;
    use github_scbot_ghapi::adapter::GifResponse;
    use github_scbot_ghapi::adapter::MediaObject;
    use maplit::hashmap;

    use super::*;
    use crate::commands::CommandContextTest;

    #[actix_rt::test]
    async fn test_valid() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.api_adapter
            .expect_gif_search()
            .times(1)
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

    #[actix_rt::test]
    async fn test_invalid() -> Result<()> {
        let mut ctx = CommandContextTest::new();

        ctx.api_adapter
            .expect_gif_search()
            .times(1)
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
