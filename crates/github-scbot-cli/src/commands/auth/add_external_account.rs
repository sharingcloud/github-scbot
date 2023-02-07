use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::AddExternalAccountUseCase;

/// Create external account
#[derive(Parser)]
pub(crate) struct AuthAddExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthAddExternalAccountCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        AddExternalAccountUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "External account '{}' created.", self.username)?;

        Ok(())
    }
}
