use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAllExternalAccountRightsUseCase;

/// Remove all rights from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAllExternalAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveAllExternalAccountRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAllExternalAccountRightsUseCase {
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
