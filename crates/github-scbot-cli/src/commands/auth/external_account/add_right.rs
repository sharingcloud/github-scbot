use std::io::Write;

use clap::Parser;
use github_scbot_domain::use_cases::auth::AddExternalAccountRightUseCase;
use github_scbot_domain_models::RepositoryPath;

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
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        AddExternalAccountRightUseCase {
            db_service: ctx.db_service.as_ref(),
        }
        .run(&self.repository_path, &self.username)
        .await?;

        writeln!(
            ctx.writer,
            "Right added to repository '{}' for external account '{}'.",
            self.repository_path, self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_database_interface::DbService;
    use github_scbot_domain_models::{ExternalAccount, Repository};

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
