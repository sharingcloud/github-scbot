//! Pull request commands.

use github_scbot_api::adapter::IAPIAdapter;
use github_scbot_conf::Config;
use github_scbot_database::models::{IDatabaseAdapter, RepositoryModel};
use github_scbot_logic::{pulls::synchronize_pull_request, status::update_pull_request_status};
use github_scbot_redis::IRedisAdapter;

use super::errors::Result;

pub(crate) async fn show_pull_request(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    number: u64,
) -> Result<()> {
    let (pr, _repo) = db_adapter
        .pull_request()
        .get_from_repository_path_and_number(repository_path, number)
        .await?;
    println!(
        "Accessing pull request #{} on repository {}",
        number, repository_path
    );
    println!("{:#?}", pr);

    Ok(())
}

pub(crate) async fn list_pull_requests(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let prs = db_adapter
        .pull_request()
        .list_from_repository_path(repository_path)
        .await?;
    if prs.is_empty() {
        println!("No PR found from repository '{}'.", repository_path);
    } else {
        for pr in prs {
            println!("- #{}: {}", pr.get_number(), pr.name);
        }
    }

    Ok(())
}

pub(crate) async fn sync_pull_request(
    config: &Config,
    api_adapter: &impl IAPIAdapter,
    db_adapter: &dyn IDatabaseAdapter,
    redis_adapter: &dyn IRedisAdapter,
    repository_path: String,
    number: u64,
) -> Result<()> {
    let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(&repository_path)?;
    let (mut pr, sha) =
        synchronize_pull_request(config, api_adapter, db_adapter, owner, name, number).await?;
    let repo = db_adapter
        .repository()
        .get_from_owner_and_name(owner, name)
        .await?;
    update_pull_request_status(api_adapter, db_adapter, redis_adapter, &repo, &mut pr, &sha)
        .await?;

    println!(
        "Pull request #{} from {} updated from GitHub.",
        number, repository_path
    );
    Ok(())
}
