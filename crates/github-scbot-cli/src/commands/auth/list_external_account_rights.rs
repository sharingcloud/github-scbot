use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::ListExternalAccountRightsUseCase;

/// List rights for account
#[derive(Parser)]
pub(crate) struct AuthListExternalAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthListExternalAccountRightsCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let repositories = ListExternalAccountRightsUseCase {
            username: &self.username,
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        if repositories.is_empty() {
            writeln!(
                ctx.writer,
                "No right found from account '{}'.",
                self.username
            )?;
        } else {
            writeln!(ctx.writer, "Rights from account '{}':", self.username)?;
            for repo in repositories {
                writeln!(ctx.writer, "- {}/{}", repo.owner, repo.name)?;
            }
        }

        Ok(())
    }
}
