use std::io::Write;

use crate::Result;
use clap::Parser;
use github_scbot_domain::use_cases::auth::CreateExternalTokenUseCaseInterface;

/// Create external token
#[derive(Parser)]
pub(crate) struct AuthCreateExternalTokenCommand {
    /// Account username
    pub username: String,
}

impl AuthCreateExternalTokenCommand {
    pub async fn run<W: Write>(
        self,
        mut writer: W,
        use_case: &dyn CreateExternalTokenUseCaseInterface,
    ) -> Result<()> {
        let token = use_case.run().await?;

        writeln!(writer, "{}", token)?;

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
    impl CreateExternalTokenUseCaseInterface for Impl {
        async fn run(&self) -> Result<String, DomainError> {
            Ok("hello".into())
        }
    }

    #[actix_rt::test]
    async fn test() -> Result<()> {
        let mut buf = Vec::new();
        let cmd = AuthCreateExternalTokenCommand {
            username: "me".into(),
        };
        cmd.run(&mut buf, &Impl).await?;

        assert_eq!(buffer_to_string(buf), "hello\n");

        Ok(())
    }
}
