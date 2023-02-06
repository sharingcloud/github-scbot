use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAllAccountRightsUseCase;

/// Remove all rights from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveAccountRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAllAccountRightsUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(
            ctx.writer,
            "All rights removed from account '{}'.",
            self.username
        )?;

        Ok(())
    }
}
