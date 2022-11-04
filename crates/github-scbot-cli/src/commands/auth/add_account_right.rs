use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_core::types::repository::RepositoryPath;
use github_scbot_domain::use_cases::auth::AddAccountRightUseCaseInterface;

/// Add right to account
#[derive(Parser)]
pub(crate) struct AuthAddAccountRightCommand {
    /// Account username
    pub username: String,
    /// Repository path (e.g. `MyOrganization/my-project`)
    pub repository_path: RepositoryPath,
}

impl AuthAddAccountRightCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn AddAccountRightUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(
            writer,
            "Right added to repository '{}' for account '{}'.",
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

    struct Impl;

    #[async_trait(?Send)]
    impl AddAccountRightUseCaseInterface for Impl {
        async fn run(&self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut buf = Vec::new();
        let cmd = AuthAddAccountRightCommand {
            repository_path: "owner/name".try_into().unwrap(),
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;

        assert_eq!(
            buffer_to_string(buf),
            "Right added to repository 'owner/name' for account 'me'.\n"
        );

        Ok(())
    }
}
