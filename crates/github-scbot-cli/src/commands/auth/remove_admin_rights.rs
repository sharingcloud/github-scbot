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
            db_service: ctx.db_service.as_mut(),
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "remove-admin-rights", "me"]).await,
            "Account 'me' added/edited without admin rights.\n"
        );

        Ok(())
    }
}
