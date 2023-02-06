use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::AddAdminRightUseCase;

/// Add admin rights to account
#[derive(Parser)]
pub(crate) struct AuthAddAdminRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthAddAdminRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        AddAdminRightUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(
            ctx.writer,
            "Account '{}' added/edited with admin rights.",
            self.username
        )?;

        Ok(())
    }
}
