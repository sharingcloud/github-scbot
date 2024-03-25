use clap::Parser;
use prbot_core::use_cases::auth::ListExternalAccounts;

use crate::{commands::CommandContext, Result};

/// List external accounts
#[derive(Parser)]
pub(crate) struct AuthExternalAccountListCommand;

impl AuthExternalAccountListCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        let accounts = ListExternalAccounts.run(&ctx.as_core_context()).await?;

        if accounts.is_empty() {
            writeln!(ctx.writer.write().await, "No external account found.")?;
        } else {
            writeln!(ctx.writer.write().await, "External accounts:")?;
            for account in accounts {
                writeln!(ctx.writer.write().await, "- {}", account.username)?;
            }
        }

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
    async fn run_no_accounts() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "external-accounts", "list"]).await,
            "No external account found.\n"
        );

        Ok(())
    }

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
            test_command(ctx, &["auth", "external-accounts", "list"]).await,
            "External accounts:\n- me\n"
        );

        Ok(())
    }
}
