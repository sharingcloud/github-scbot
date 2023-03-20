use std::collections::{hash_map::Entry, HashMap};

use github_scbot_domain_models::{ChecksStatus, RepositoryPath};
use github_scbot_ghapi_interface::{
    types::{GhCheckConclusion, GhCheckRun},
    ApiService,
};

use crate::Result;

pub struct DetermineChecksStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

impl<'a> DetermineChecksStatusUseCase<'a> {
    #[tracing::instrument(
        skip(self),
        fields(repository_path, commit_sha, wait_for_initial_checks),
        ret
    )]
    pub async fn run(
        &self,
        repository_path: &RepositoryPath,
        commit_sha: &str,
        wait_for_initial_checks: bool,
    ) -> Result<ChecksStatus> {
        // Get upstream checks
        let check_runs = self
            .api_service
            .check_runs_list(repository_path.owner(), repository_path.name(), commit_sha)
            .await?;

        // Extract status
        if check_runs.is_empty() {
            if wait_for_initial_checks {
                Ok(ChecksStatus::Waiting)
            } else {
                Ok(ChecksStatus::Skipped)
            }
        } else {
            Ok(filter_and_merge_check_runs(
                &check_runs,
                wait_for_initial_checks,
            ))
        }
    }
}

/// Filter and merge check suites.
fn filter_and_merge_check_runs(
    check_runs: &[GhCheckRun],
    wait_for_initial_checks: bool,
) -> ChecksStatus {
    let filtered = filter_last_check_runs(check_runs);
    marge_check_run_statuses(&filtered, wait_for_initial_checks)
}

/// Filter last check runs, using the name of the check run to dedupe.
fn filter_last_check_runs(check_runs: &[GhCheckRun]) -> Vec<GhCheckRun> {
    let mut map: HashMap<String, GhCheckRun> = HashMap::new();
    // Only fetch GitHub Actions statuses
    for check_run in check_runs.iter().filter(|s| s.app.slug == "github-actions") {
        if let Entry::Vacant(e) = map.entry(check_run.name.clone()) {
            e.insert(check_run.clone());
        } else {
            let entry = map.get_mut(&check_run.name).unwrap();
            if entry.started_at < check_run.started_at {
                *entry = check_run.clone();
            }
        }
    }

    map.into_values().collect()
}

fn marge_check_run_statuses(
    check_runs: &[GhCheckRun],
    wait_for_initial_checks: bool,
) -> ChecksStatus {
    let initial = if wait_for_initial_checks {
        ChecksStatus::Waiting
    } else {
        ChecksStatus::Skipped
    };

    check_runs
        .iter()
        .fold(None, |acc, s| match (&acc, &s.conclusion) {
            // Already failed, or current check suite is failing
            (Some(ChecksStatus::Fail), _) | (_, Some(GhCheckConclusion::Failure)) => {
                Some(ChecksStatus::Fail)
            }
            // No status or checks already pass, and current check suite pass
            (None | Some(ChecksStatus::Pass), Some(GhCheckConclusion::Success)) => {
                Some(ChecksStatus::Pass)
            }
            // No conclusion for current check suite
            (_, None) => Some(ChecksStatus::Waiting),
            // Keep same status
            (_, _) => acc,
        })
        .unwrap_or(initial)
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::types::{GhApplication, GhCheckStatus};
    use time::{Duration, OffsetDateTime};

    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    pub fn test_merge_check_suite_statuses() {
        // No check suite, no need to wait
        assert_eq!(
            filter_and_merge_check_runs(&[], false),
            ChecksStatus::Skipped
        );

        // No check suite, but with initial checks wait
        assert_eq!(
            filter_and_merge_check_runs(&[], true),
            ChecksStatus::Waiting
        );

        let base_run = GhCheckRun {
            app: GhApplication {
                slug: "github-actions".into(),
                ..GhApplication::default()
            },
            ..GhCheckRun::default()
        };

        // Should wait on queued status
        assert_eq!(
            filter_and_merge_check_runs(
                &[GhCheckRun {
                    status: GhCheckStatus::Queued,
                    conclusion: None,
                    ..base_run.clone()
                }],
                false
            ),
            ChecksStatus::Waiting
        );

        // Ignore unsupported apps
        assert_eq!(
            filter_and_merge_check_runs(
                &[GhCheckRun {
                    status: GhCheckStatus::Queued,
                    app: GhApplication {
                        slug: "toto".into(),
                        ..GhApplication::default()
                    },
                    ..GhCheckRun::default()
                }],
                false
            ),
            ChecksStatus::Skipped
        );

        // Success
        assert_eq!(
            filter_and_merge_check_runs(
                &[GhCheckRun {
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Success),
                    ..base_run.clone()
                }],
                false
            ),
            ChecksStatus::Pass
        );

        // Success with skipped
        assert_eq!(
            filter_and_merge_check_runs(
                &[
                    GhCheckRun {
                        name: "Foo".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_run.clone()
                    },
                    GhCheckRun {
                        name: "Bar".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_run.clone()
                    }
                ],
                false,
            ),
            ChecksStatus::Pass
        );

        // Success with queued
        assert_eq!(
            filter_and_merge_check_runs(
                &[
                    GhCheckRun {
                        name: "Foo".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_run.clone()
                    },
                    GhCheckRun {
                        name: "Bar".into(),
                        status: GhCheckStatus::Queued,
                        conclusion: None,
                        ..base_run.clone()
                    }
                ],
                false
            ),
            ChecksStatus::Waiting
        );

        // One failing check make the status fail
        assert_eq!(
            filter_and_merge_check_runs(
                &[
                    GhCheckRun {
                        name: "Foo".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        ..base_run.clone()
                    },
                    GhCheckRun {
                        name: "Bar".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_run.clone()
                    }
                ],
                false,
            ),
            ChecksStatus::Fail
        );

        // Two GitHub actions at different moments
        let now = OffsetDateTime::now_utc();
        assert_eq!(
            filter_and_merge_check_runs(
                &[
                    GhCheckRun {
                        name: "Foo".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        started_at: now + Duration::hours(1),
                        ..base_run.clone()
                    },
                    GhCheckRun {
                        name: "Foo".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        started_at: now,
                        ..base_run.clone()
                    },
                    GhCheckRun {
                        name: "Bar".into(),
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_run
                    }
                ],
                false,
            ),
            ChecksStatus::Pass
        );
    }
}
