use clap::Parser;
use prbot_core::use_cases::auth::RemoveAdminRight;

use crate::{commands::CommandContext, Result};

/// Remove admin rights from account
#[derive(Parser)]
pub(crate) struct AuthAdminRemoveCommand {
    /// Account username
    pub username: String,
}

impl AuthAdminRemoveCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        RemoveAdminRight
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(
            ctx.writer.write().await,
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
