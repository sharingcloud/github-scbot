use anyhow::{anyhow, Result};
use github_scbot_database::{DbService, PullRequest, Repository};

pub struct CliDbExt;

impl CliDbExt {
    pub async fn get_existing_repository(
        db_service: &mut dyn DbService,
        owner: &str,
        name: &str,
    ) -> Result<Repository> {
        let opt = db_service.repositories_get(owner, name).await?;

        match opt {
            Some(s) => Ok(s),
            None => Err(anyhow!("Unknown repository '{}/{}'", owner, name)),
        }
    }

    pub async fn get_existing_pull_request(
        db_service: &mut dyn DbService,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<PullRequest> {
        let opt = db_service.pull_requests_get(owner, name, number).await?;

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
