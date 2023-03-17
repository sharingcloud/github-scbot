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
    pub async fn run<W: Write>(self, ctx: &mut CommandContext<W>) -> Result<()> {
        AddAdminRightUseCase {
            username: &self.username,
            db_service: ctx.db_service.as_mut(),
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();

        assert_eq!(
            test_command(ctx, &["auth", "add-admin-rights", "me"]).await,
            "Account 'me' added/edited with admin rights.\n"
        );

        Ok(())
    }
}
