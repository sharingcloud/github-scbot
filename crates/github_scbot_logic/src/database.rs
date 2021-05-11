//! Database module.

use github_scbot_api::labels::set_step_label;
use github_scbot_conf::Config;
use github_scbot_database::models::{PullRequestModel, RepositoryModel};

use crate::errors::Result;

/// Apply pull request step.
pub async fn apply_pull_request_step(
    config: &Config,
    repository_model: &RepositoryModel,
    pr_model: &PullRequestModel,
) -> Result<()> {
    set_step_label(
        config,
        &repository_model.owner,
        &repository_model.name,
        pr_model.get_number(),
        pr_model.get_step_label(),
    )
    .await
    .map_err(Into::into)
}
