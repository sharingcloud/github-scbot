use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::use_cases::auth::RemoveAccountRightUseCase;

/// Remove right from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthRemoveAccountRightCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAccountRightUseCase {
            username: self.username.clone(),
            repository_path: self.repository_path.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
        .await?;

        writeln!(
            ctx.writer,
            "Right removed to repository '{}' for account '{}'.",
            self.repository_path, self.username
        )?;

        Ok(())
    }
}
