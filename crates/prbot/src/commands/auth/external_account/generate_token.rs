use clap::Parser;
use prbot_core::use_cases::auth::GenerateExternalAccountToken;

use crate::{commands::CommandContext, Result};

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthExternalAccountGenerateTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountGenerateTokenCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        let token = GenerateExternalAccountToken
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(ctx.writer.write().await, "{}", token)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::ExternalAccount;

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        ctx.db_service
            .external_accounts_create(
                ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        assert!(
            test_command(ctx, &["auth", "external-accounts", "generate-token", "me"])
                .await
                .starts_with("ey")
        );

        Ok(())
    }
}
