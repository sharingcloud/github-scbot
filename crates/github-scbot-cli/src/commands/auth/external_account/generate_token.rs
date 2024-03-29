use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::GenerateExternalAccountTokenUseCase;

use crate::{commands::CommandContext, Result};

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthExternalAccountGenerateTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthExternalAccountGenerateTokenCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        let token = GenerateExternalAccountTokenUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run(&self.username)
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

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
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
            test_command(ctx, &["auth", "external-accounts", "generate-token", "me"])
                .await
                .starts_with("ey")
        );

        Ok(())
    }
}
