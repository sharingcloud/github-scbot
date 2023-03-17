use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveExternalAccountUseCase;

use crate::{commands::CommandContext, Result};

/// Remove external account
#[derive(Parser)]
pub(crate) struct AuthRemoveExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveExternalAccountCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        RemoveExternalAccountUseCase {
            username: self.username.clone(),
            db_service: ctx.db_service.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "External account '{}' removed.", self.username)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::ExternalAccount;

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut ctx = CommandContextTest::new();
        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        assert_eq!(
            test_command(ctx, &["auth", "remove-external-account", "me"]).await,
            "External account 'me' removed.\n"
        );

        Ok(())
    }
}
