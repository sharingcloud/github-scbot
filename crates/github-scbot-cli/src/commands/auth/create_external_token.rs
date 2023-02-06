use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::CreateExternalTokenUseCase;

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthCreateExternalTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthCreateExternalTokenCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let token = CreateExternalTokenUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "{}", token)?;

        Ok(())
    }
}
