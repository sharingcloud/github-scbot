use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveAllAccountRightsUseCaseInterface;

/// Remove all rights from account
#[derive(Parser)]
pub(crate) struct AuthRemoveAccountRightsCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveAccountRightsCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn RemoveAllAccountRightsUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(
            writer,
            "All rights removed from account '{}'.",
            self.username
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
        impl RemoveAllAccountRightsUseCaseInterface for Impl {
            async fn run(&self) -> Result<(), DomainError> {
                Ok(())
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthRemoveAccountRightsCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(
            buffer_to_string(buf),
            "All rights removed from account 'me'.\n"
        );

        Ok(())
    }
}
