use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::AddExternalAccountUseCase;

use crate::{commands::CommandContext, Result};

/// Create external account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountAddCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountAddCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        AddExternalAccountUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run(&self.username)
        .await?;

        writeln!(ctx.writer, "External account '{}' created.", self.username)?;

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
            test_command(ctx, &["auth", "external-accounts", "add", "me"]).await,
            "External account 'me' created.\n"
        );

        Ok(())
    }
}
