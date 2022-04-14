//! Reviews module.

use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::reviews::GhReviewEvent;

use crate::{status::StatusLogic, Result};

/// Handle GitHub pull request review event.
pub async fn handle_review_event(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhReviewEvent,
) -> Result<()> {
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;
    let pr_number = event.pull_request.number;

    // Detect required reviews
    if let Some(_) = db_adapter
        .pull_requests()
        .get(repo_owner, repo_name, pr_number)
        .await?
    {
        let upstream_pr = api_adapter
            .pulls_get(repo_owner, repo_name, pr_number)
            .await?;

        StatusLogic::update_pull_request_status(
            api_adapter,
            db_adapter,
            redis_adapter,
            repo_owner,
            repo_name,
            pr_number,
            &upstream_pr,
        )
        .await?;
    }

    Ok(())
}
