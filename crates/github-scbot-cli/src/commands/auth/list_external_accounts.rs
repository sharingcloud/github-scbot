use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListExternalAccountsUseCase;

/// List external accounts
#[derive(Parser)]
pub(crate) struct AuthListExternalAccountsCommand;

impl AuthListExternalAccountsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let accounts = ListExternalAccountsUseCase {
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        if accounts.is_empty() {
            writeln!(ctx.writer, "No external account found.")?;
        } else {
            writeln!(ctx.writer, "External accounts:")?;
            for account in accounts {
                writeln!(ctx.writer, "- {}", account.username())?;
            }
        }

        Ok(())
    }
}
