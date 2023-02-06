//! Reviews module.

use github_scbot_core::types::reviews::GhReviewEvent;
use github_scbot_database::DbServiceAll;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use crate::{status::StatusLogic, Result};

/// Handle GitHub pull request review event.
#[tracing::instrument(
    skip_all,
    fields(
        repository_path = %event.repository.full_name,
        pr_number = event.pull_request.number,
        reviewer = %event.review.user.login,
        state = ?event.review.state
    )
)]
pub async fn handle_review_event(
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbServiceAll,
    redis_adapter: &dyn RedisService,
    event: GhReviewEvent,
) -> Result<()> {
    let repo_owner = &event.repository.owner.login;
    let repo_name = &event.repository.name;
    let pr_number = event.pull_request.number;

    // Detect required reviews
    if db_adapter
        .pull_requests_get(repo_owner, repo_name, pr_number)
        .await?
        .is_some()
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
