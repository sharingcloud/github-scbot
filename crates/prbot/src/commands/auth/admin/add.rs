use clap::Parser;
use prbot_core::use_cases::auth::AddAdminRight;

use crate::{commands::CommandContext, Result};

/// Add admin rights to account
#[derive(Parser)]
pub(crate) struct AuthAdminAddCommand {
    /// Account username
    pub username: String,
}

impl AuthAdminAddCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        AddAdminRight
            .run(&ctx.as_core_context(), &self.username)
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Account '{}' added/edited with admin rights.",
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
            test_command(ctx, &["auth", "admins", "add", "me"]).await,
            "Account 'me' added/edited with admin rights.\n"
        );

        Ok(())
    }
}
