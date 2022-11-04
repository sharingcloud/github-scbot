use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::CreateExternalAccountUseCaseInterface;

/// Create external account
#[derive(Parser)]
pub(crate) struct AuthCreateExternalAccountCommand {
    /// Account username
    pub username: String,
}

impl AuthCreateExternalAccountCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn CreateExternalAccountUseCaseInterface,
    ) -> Result<()> {
        use_case.run().await?;

        writeln!(writer, "External account '{}' created.", self.username)?;

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
    impl CreateExternalAccountUseCaseInterface for Impl {
        async fn run(&self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut buf = Vec::new();
        let cmd = AuthCreateExternalAccountCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;

        assert_eq!(buffer_to_string(buf), "External account 'me' created.\n");

        Ok(())
    }
}
