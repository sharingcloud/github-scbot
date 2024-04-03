use clap::Parser;
use prbot_core::use_cases::auth::AddExternalAccount;

use crate::{commands::CommandContext, Result};

/// Create external account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountAddCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountAddCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        AddExternalAccount
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "External account '{}' created.",
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "external-accounts", "add", "me"]).await,
            "External account 'me' created.\n"
        );

        Ok(())
    }
}
