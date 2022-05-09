use crate::errors::DatabaseSnafu;
use crate::Result;
use github_scbot_database2::{
    ExternalAccount, ExternalAccountDB, PullRequest, PullRequestDB, Repository, RepositoryDB,
};
use snafu::{whatever, ResultExt};

pub struct CliDbExt;

impl CliDbExt {
    pub async fn get_existing_repository(
        repository_db: &mut dyn RepositoryDB,
        owner: &str,
        name: &str,
    ) -> Result<Repository> {
        let opt = repository_db
            .get(owner, name)
            .await
            .context(DatabaseSnafu)?;

        match opt {
            Some(s) => Ok(s),
            None => whatever!("Unknown repository '{}/{}'", owner, name),
        }
    }

    pub async fn get_existing_pull_request(
        pull_request_db: &mut dyn PullRequestDB,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<PullRequest> {
        let opt = pull_request_db
            .get(owner, name, number)
            .await
            .context(DatabaseSnafu)?;

        match opt {
            Some(p) => Ok(p),
            None => whatever!(
                "Unknown pull request #{} for repository '{}/{}'",
                number,
                owner,
                name
            ),
        }
    }

    pub async fn get_existing_external_account(
        external_account_db: &mut dyn ExternalAccountDB,
        username: &str,
    ) -> Result<ExternalAccount> {
        let opt = external_account_db
            .get(username)
            .await
            .context(DatabaseSnafu)?;

        match opt {
            Some(e) => Ok(e),
            None => whatever!("Unknown external account '{}'", username),
        }
    }
}
