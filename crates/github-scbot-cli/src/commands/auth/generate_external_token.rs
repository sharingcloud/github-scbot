use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::GenerateExternalTokenUseCase;

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthGenerateExternalTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthGenerateExternalTokenCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let token = GenerateExternalTokenUseCase {
            username: self.username.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "{}", token)?;

        Ok(())
    }
}
