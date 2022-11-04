use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::RemoveExternalAccountUseCaseInterface;

/// Remove external account
#[derive(Parser)]
pub(crate) struct AuthRemoveExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthRemoveExternalAccountCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn RemoveExternalAccountUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(writer, "External account '{}' removed.", self.username)?;

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
        impl RemoveExternalAccountUseCaseInterface for Impl {
            async fn run(&self) -> Result<(), DomainError> {
                Ok(())
            }
        }

        let mut buf = Vec::new();
        let cmd = AuthRemoveExternalAccountCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;
        assert_eq!(buffer_to_string(buf), "External account 'me' removed.\n");

        Ok(())
    }
}
