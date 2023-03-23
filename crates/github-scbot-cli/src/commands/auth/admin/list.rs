use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::ListAdminAccountsUseCase;

use crate::{commands::CommandContext, Result};

/// List admin accounts
#[derive(Parser)]
pub(crate) struct AuthAdminListCommand;

impl AuthAdminListCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let accounts = ListAdminAccountsUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run()
        .await?;

        if accounts.is_empty() {
            writeln!(ctx.writer, "No admin account found.")?;
        } else {
            writeln!(ctx.writer, "Admin accounts:")?;
            for account in accounts {
                writeln!(ctx.writer, "- {}", account.username)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::Account;

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
