use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_domain::use_cases::auth::GenerateExternalAccountTokenUseCase;

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthGenerateExternalAccountTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthGenerateExternalAccountTokenCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let token = GenerateExternalAccountTokenUseCase {
            username: &self.username,
            db_service: ctx.db_service.as_mut(),
        }
        .run()
        .await?;

        writeln!(ctx.writer, "{}", token)?;

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
            .external_accounts_create(
                ExternalAccount {
                    username: "me".into(),
                    ..Default::default()
                }
                .with_generated_keys(),
            )
            .await?;

        assert!(
            test_command(ctx, &["auth", "generate-external-account-token", "me"])
                .await
                .starts_with("ey")
        );

        Ok(())
    }
}
