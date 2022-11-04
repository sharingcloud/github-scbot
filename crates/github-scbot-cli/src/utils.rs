use anyhow::{anyhow, Result};
use github_scbot_database::{
    ExternalAccount, ExternalAccountDB, PullRequest, PullRequestDB, Repository, RepositoryDB,
};

pub struct CliDbExt;

impl CliDbExt {
    pub async fn get_existing_repository(
        repository_db: &mut dyn RepositoryDB,
        owner: &str,
        name: &str,
    ) -> Result<Repository> {
        let opt = repository_db.get(owner, name).await?;

        match opt {
            Some(s) => Ok(s),
            None => Err(anyhow!("Unknown repository '{}/{}'", owner, name)),
        }
    }

    pub async fn get_existing_pull_request(
        pull_request_db: &mut dyn PullRequestDB,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<PullRequest> {
        let opt = pull_request_db.get(owner, name, number).await?;

        match opt {
            Some(p) => Ok(p),
            None => Err(anyhow!(
                "Unknown pull request #{} for repository '{}/{}'",
                number,
                owner,
                name
            )),
        }
    }
}
