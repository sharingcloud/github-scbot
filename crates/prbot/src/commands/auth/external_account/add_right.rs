use clap::Parser;
use prbot_core::use_cases::auth::AddExternalAccountRight;
use prbot_models::RepositoryPath;

use crate::{commands::CommandContext, Result};

/// Add right to account
#[derive(Parser)]
pub(crate) struct AuthExternalAccountAddRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthExternalAccountAddRightCommand {
    pub async fn run(self, ctx: CommandContext) -> Result<()> {
        AddExternalAccountRight
            .run(
                &ctx.as_core_context(),
                &self.repository_path,
                &self.username,
            )
            .await?;

        writeln!(
            ctx.writer.write().await,
            "Right added to repository '{}' for external account '{}'.",
            self.repository_path,
            self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_models::{ExternalAccount, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[tokio::test]
    async fn run() {
        let ctx = CommandContextTest::new();
        ctx.db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "repo".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        ctx.db_service
            .external_accounts_create(ExternalAccount {
                username: "me".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let result = test_command(
            ctx,
            &["auth", "external-accounts", "add-right", "me", "me/repo"],
        )
        .await;
        assert_eq!(
            result,
            "Right added to repository 'me/repo' for external account 'me'.\n"
        );
    }
}
