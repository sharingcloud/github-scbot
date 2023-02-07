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
