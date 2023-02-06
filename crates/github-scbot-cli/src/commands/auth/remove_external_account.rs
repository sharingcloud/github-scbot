use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveExternalAccountUseCase;

/// Remove external account
#[derive(Parser)]
pub(crate) struct AuthRemoveExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveExternalAccountCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveExternalAccountUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "External account '{}' removed.", self.username)?;

        Ok(())
    }
}
