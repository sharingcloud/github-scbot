use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::ListExternalAccountsUseCase;

use crate::{commands::CommandContext, Result};

/// List external accounts
#[derive(Parser)]
pub(crate) struct AuthExternalAccountListCommand;

impl AuthExternalAccountListCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let accounts = ListExternalAccountsUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run()
        .await?;

        if accounts.is_empty() {
            writeln!(ctx.writer, "No external account found.")?;
        } else {
            writeln!(ctx.writer, "External accounts:")?;
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
    use github_scbot_domain_models::ExternalAccount;

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
