use std::collections::{hash_map::Entry, HashMap};

use async_trait::async_trait;
use prbot_ghapi_interface::types::{GhCommitStatusItem, GhCommitStatusState};
use prbot_models::{ChecksStatus, RepositoryPath};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait DetermineCommitStatusInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        commit_sha: &str,
        wait_for_initial_checks: bool,
    ) -> Result<ChecksStatus>;
}

#[derive(Component)]
#[shaku(interface = DetermineCommitStatusInterface)]
pub(crate) struct DetermineCommitStatus;

#[async_trait]
impl DetermineCommitStatusInterface for DetermineCommitStatus {
    #[tracing::instrument(
        skip(self, ctx),
        fields(api_service, repository_path, commit_sha, wait_for_initial_checks),
        ret
    )]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        commit_sha: &str,
        wait_for_initial_checks: bool,
    ) -> Result<ChecksStatus> {
        // Get upstream statuses
        let commit_statuses = ctx
            .api_service
            .commit_statuses_combined(repository_path.owner(), repository_path.name(), commit_sha)
            .await?;

        // Extract status
        if commit_statuses.items.is_empty() {
            if wait_for_initial_checks {
                Ok(ChecksStatus::Waiting)
            } else {
                Ok(ChecksStatus::Skipped)
            }
        } else {
            // TODO
            Ok(filter_and_merge_statuses(
                &commit_statuses.items,
                wait_for_initial_checks,
            ))
        }
    }
}

/// Filter and merge statuses.
fn filter_and_merge_statuses(
    status_items: &[GhCommitStatusItem],
    wait_for_initial_checks: bool,
) -> ChecksStatus {
    let filtered = filter_statuses(status_items);
    merge_statuses(&filtered, wait_for_initial_checks)
}

/// Filter last check runs, using the name of the check run to dedupe.
fn filter_statuses(check_runs: &[GhCommitStatusItem]) -> Vec<GhCommitStatusItem> {
    let mut map: HashMap<String, GhCommitStatusItem> = HashMap::new();
    // Ignore prbot status
    for item in check_runs.iter().filter(|s| s.context != "Validation") {
        if let Entry::Vacant(e) = map.entry(item.context.clone()) {
            e.insert(item.clone());
        } else {
            let entry = map.get_mut(&item.context).unwrap();
            if entry.updated_at < item.updated_at {
                *entry = item.clone();
            }
        }
    }

    map.into_values().collect()
}

fn merge_statuses(
    status_items: &[GhCommitStatusItem],
    wait_for_initial_checks: bool,
) -> ChecksStatus {
    let initial = if wait_for_initial_checks {
        ChecksStatus::Waiting
    } else {
        ChecksStatus::Skipped
    };

    status_items
        .iter()
        .fold(None, |acc, s| match (&acc, &s.state) {
            // Already failed, or current check suite is failing
            (Some(ChecksStatus::Fail), _)
            | (_, GhCommitStatusState::Failure)
            | (_, GhCommitStatusState::Error) => Some(ChecksStatus::Fail),
            // No status or checks already pass, and current check suite pass
            (None | Some(ChecksStatus::Pass), GhCommitStatusState::Success) => {
                Some(ChecksStatus::Pass)
            }
            // No conclusion for current check suite
            (_, GhCommitStatusState::Pending) => Some(ChecksStatus::Waiting),
            // Keep same status
            (_, _) => acc,
        })
        .unwrap_or(initial)
}
