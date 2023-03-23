use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAdminRightUseCase;

use crate::{commands::CommandContext, Result};

/// Remove admin rights from account
#[derive(Parser)]
pub(crate) struct AuthAdminRemoveCommand {
    /// Account username
    pub username: String,
}

impl AuthAdminRemoveCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveAdminRightUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run(&self.username)
        .await?;

        writeln!(
            ctx.writer,
            "Account '{}' added/edited without admin rights.",
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "admins", "remove", "me"]).await,
            "Account 'me' added/edited without admin rights.\n"
        );

        Ok(())
    }
}
