use clap::Parser;
use prbot_core::use_cases::auth::ListAdminAccounts;

use crate::{commands::CommandContext, Result};

/// List admin accounts
#[derive(Parser)]
pub(crate) struct AuthAdminListCommand;

impl AuthAdminListCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        let accounts = ListAdminAccounts.run(&ctx.as_core_context()).await?;

        if accounts.is_empty() {
            writeln!(ctx.writer.write().await, "No admin account found.")?;
        } else {
            writeln!(ctx.writer.write().await, "Admin accounts:")?;
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
    use prbot_models::Account;

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run_no_accounts() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "admins", "list"]).await,
            "No admin account found.\n"
        );

        Ok(())
    }

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        ctx.db_service
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["auth", "admins", "list"]).await,
            "Admin accounts:\n- me\n"
        );

        Ok(())
    }
}
