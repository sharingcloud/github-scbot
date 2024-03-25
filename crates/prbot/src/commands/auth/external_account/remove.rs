use clap::Parser;
use prbot_core::use_cases::auth::RemoveExternalAccount;

use crate::{commands::CommandContext, Result};

/// Remove external account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountRemoveCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountRemoveCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        RemoveExternalAccount
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "External account '{}' removed.",
            self.username
        )?;

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
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["auth", "external-accounts", "remove", "me"]).await,
            "External account 'me' removed.\n"
        );

        Ok(())
    }
}
