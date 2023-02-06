use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAdminRightUseCase;

/// Remove admin rights from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAdminRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveAdminRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAdminRightUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(
            ctx.writer,
            "Account '{}' added/edited without admin rights.",
            self.username
        )?;

        Ok(())
    }
}
