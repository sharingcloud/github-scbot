use github_scbot_database2::{
    ExternalAccount, ExternalAccountDB, PullRequest, PullRequestDB, Repository, RepositoryDB,
};
use github_scbot_sentry::eyre::{self, eyre::eyre};

pub struct CliDbExt;

impl CliDbExt {
    pub async fn get_existing_repository(
        repository_db: &mut dyn RepositoryDB,
        owner: &str,
        name: &str,
    ) -> eyre::Result<Repository> {
        repository_db
            .get(owner, name)
            .await?
            .ok_or_else(|| eyre!("Unknown repository '{}/{}'", owner, name))
    }

    pub async fn get_existing_pull_request(
        pull_request_db: &mut dyn PullRequestDB,
        owner: &str,
        name: &str,
        number: u64,
    ) -> eyre::Result<PullRequest> {
        pull_request_db
            .get(owner, name, number)
            .await?
            .ok_or_else(|| {
                eyre!(
                    "Unknown pull request #{} for repository '{}/{}'",
                    number,
                    owner,
                    name
                )
            })
    }

    pub async fn get_existing_external_account(
        external_account_db: &mut dyn ExternalAccountDB,
        username: &str,
    ) -> eyre::Result<ExternalAccount> {
        external_account_db
            .get(username)
            .await?
            .ok_or_else(|| eyre!("Unknown external account '{}'", username))
    }
}
