use std::io::Write;

use crate::{commands::CommandContext, Result};
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::use_cases::auth::AddExternalAccountRightUseCase;

/// Add right to account
#[derive(Parser)]
pub(crate) struct AuthAddExternalAccountRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthAddExternalAccountRightCommand {
    pub async fn run<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        AddExternalAccountRightUseCase {
            username: &self.username,
            repository_path: self.repository_path.clone(),
            db_service: ctx.db_adapter.as_mut(),
        }
        .run()
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
    use github_scbot_database::{DbServiceAll, ExternalAccount, Repository};

    use crate::testutils::{test_command, CommandContextTest};

    #[actix_rt::test]
    async fn command() {
        let mut ctx = CommandContextTest::new();
        ctx.db_adapter
            .repositories_create(
                Repository::builder()
                    .owner("me")
                    .name("repo")
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();

        ctx.db_adapter
            .external_accounts_create(ExternalAccount::builder().username("me").build().unwrap())
            .await
            .unwrap();

        let result = test_command(
            ctx,
            &["auth", "add-external-account-right", "me", "me/repo"],
        )
        .await;
        assert_eq!(
            result,
            "Right added to repository 'me/repo' for external account 'me'.\n"
        );
    }
}
