use clap::Parser;
use prbot_core::use_cases::auth::RemoveExternalAccountRight;
use prbot_models::RepositoryPath;

use crate::{commands::CommandContext, Result};

/// Remove right from account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountRemoveRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthExternalAccountRemoveRightCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        RemoveExternalAccountRight
            .run(
                &ctx.as_core_context(),
                &self.repository_path,
                &self.username,
            )
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Right removed to repository '{}' for external account '{}'.",
            self.repository_path,
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, ExternalAccountRight, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let ctx = CommandContextTest::new();
        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await?;

        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        ctx.db_service
            .external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(
            test_command(
                ctx,
                &[
                    "auth",
                    "external-accounts",
                    "remove-right",
                    "me",
                    "owner/name"
                ]
            )
            .await,
            "Right removed to repository 'owner/name' for external account 'me'.\n"
        );

        Ok(())
    }
}
