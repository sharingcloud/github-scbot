//! Reviews module.

use github_scbot_conf::Config;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{
    reviews::GhReviewEvent,
};

use crate::{status::StatusLogic, Result};

/// Handle GitHub pull request review event.
pub async fn handle_review_event(
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhReviewEvent,
) -> Result<()> {
    // Detect required reviews
    if let Some(pr) = db_adapter.pull_requests().get(&event.repository.owner.login, &event.repository.name, event.pull_request.number).await? {
        let repo = db_adapter.repositories().get(&event.repository.owner.login, &event.repository.name).await?.unwrap();
        let upstream_pr = api_adapter.pulls_get(repo.owner(), repo.name(), pr.number()).await?;

        StatusLogic::update_pull_request_status(
            api_adapter,
            db_adapter,
            redis_adapter,
            &repo,
            &pr,
            &upstream_pr
        )
        .await?;
    }

    Ok(())
}
