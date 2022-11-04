use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::use_cases::auth::RemoveAccountRightUseCaseInterface;

/// Remove right from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthRemoveAccountRightCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn RemoveAccountRightUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(
            writer,
            "Right removed to repository '{}' for account '{}'.",
            self.repository_path, self.username
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use github_scbot_domain::DomainError;

    use super::*;
    use crate::testutils::buffer_to_string;

    #[actix_rt::test]
    async fn test() -> Result<()> {
        struct Impl;

        #[async_trait(?Send)]
        impl RemoveAccountRightUseCaseInterface for Impl {
            async fn run(&self) -> Result<(), DomainError> {
                Ok(())
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthRemoveAccountRightCommand {
            username: "me".into(),
            repository_path: "owner/name".try_into().unwrap(),
        };
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(
            buffer_to_string(buf),
            "Right removed to repository 'owner/name' for account 'me'.\n"
        );

        Ok(())
    }
}
