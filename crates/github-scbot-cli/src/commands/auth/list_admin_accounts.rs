use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListAdminAccountsUseCase;

/// List admin accounts
#[derive(Parser)]
pub(crate) struct AuthListAdminAccountsCommand;

impl AuthListAdminAccountsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let accounts = ListAdminAccountsUseCase {
            db_service: ctx.db_adapter.as_mut(),
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

    #[actix_rt::test]
    async fn run_no_accounts() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "list-admin-accounts"]).await,
            "No admin account found.\n"
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .accounts_create(Account {
                username: "me".into(),
                is_admin: true,
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["auth", "list-admin-accounts"]).await,
            "Admin accounts:\n- me\n"
        );

        Ok(())
    }
}
