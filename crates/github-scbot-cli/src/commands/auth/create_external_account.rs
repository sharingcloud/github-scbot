use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::CreateExternalAccountUseCase;

/// Create external account
#[derive(Parser)]
pub(crate) struct AuthCreateExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthCreateExternalAccountCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        CreateExternalAccountUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "External account '{}' created.", self.username)?;

        Ok(())
    }
}
