use std::collections::{hash_map::Entry, HashMap};

use github_scbot_domain_models::ChecksStatus;
use github_scbot_ghapi_interface::{
    types::{GhCheckConclusion, GhCheckSuite},
    ApiService,
};

use crate::Result;

pub struct DetermineChecksStatusUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub commit_sha: &'a str,
    pub wait_for_initial_checks: bool,
    pub exclude_check_suite_ids: &'a [u64],
}

struct FilterCheckSuitesUseCase<'a> {
    check_suites: &'a [GhCheckSuite],
    wait_for_initial_checks: bool,
    exclude_check_suite_ids: &'a [u64],
}

impl<'a> DetermineChecksStatusUseCase<'a> {
    #[tracing::instrument(skip(self), ret)]
    pub async fn run(&mut self) -> Result<ChecksStatus> {
        // Get upstream checks
        let check_suites = self
            .api_service
            .check_suites_list(self.repo_owner, self.repo_name, self.commit_sha)
            .await?;

        // Extract status
        if check_suites.is_empty() {
            if self.wait_for_initial_checks {
                Ok(ChecksStatus::Waiting)
            } else {
                Ok(ChecksStatus::Skipped)
            }
        } else {
            let filtered = FilterCheckSuitesUseCase {
                check_suites: &check_suites,
                wait_for_initial_checks: self.wait_for_initial_checks,
                exclude_check_suite_ids: self.exclude_check_suite_ids,
            }
            .run();

            Ok(filtered)
        }
    }
}

impl<'a> FilterCheckSuitesUseCase<'a> {
    /// Filter and merge check suites.
    pub fn run(&mut self) -> ChecksStatus {
        let filtered = self.filter_last_check_suites();
        self.merge_check_suite_statuses(&filtered)
    }

    /// Filter last check suites.
    fn filter_last_check_suites(&mut self) -> Vec<GhCheckSuite> {
        let mut map: HashMap<u64, GhCheckSuite> = HashMap::new();
        // Only fetch GitHub Actions statuses
        for check_suite in self.check_suites.iter().filter(|s| {
            s.app.slug == "github-actions" && !self.exclude_check_suite_ids.contains(&s.id)
        }) {
            if let Entry::Vacant(e) = map.entry(check_suite.id) {
                e.insert(check_suite.clone());
            } else {
                let entry = map.get_mut(&check_suite.id).unwrap();
                if entry.updated_at < check_suite.updated_at {
                    *entry = check_suite.clone();
                }
            }
        }

        map.into_values().collect()
    }

    /// Merge check suite statuses.
    fn merge_check_suite_statuses(&mut self, check_suites: &[GhCheckSuite]) -> ChecksStatus {
        let initial = if self.wait_for_initial_checks {
            ChecksStatus::Waiting
        } else {
            ChecksStatus::Skipped
        };

        check_suites
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
}

#[cfg(test)]
mod tests {
    use github_scbot_domain_models::ChecksStatus;
    use github_scbot_ghapi_interface::types::{
        GhApplication, GhCheckConclusion, GhCheckStatus, GhCheckSuite,
    };
    use time::{Duration, OffsetDateTime};

    use super::FilterCheckSuitesUseCase;

    #[test]
    #[allow(clippy::too_many_lines)]
    pub fn test_merge_check_suite_statuses() {
        // No check suite, no need to wait
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[],
                exclude_check_suite_ids: &[],
                wait_for_initial_checks: false
            }
            .run(),
            ChecksStatus::Skipped
        );

        // No check suite, but with initial checks wait
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[],
                exclude_check_suite_ids: &[],
                wait_for_initial_checks: true
            }
            .run(),
            ChecksStatus::Waiting
        );

        let base_suite = GhCheckSuite {
            app: GhApplication {
                slug: "github-actions".into(),
                ..GhApplication::default()
            },
            ..GhCheckSuite::default()
        };

        // Should wait on queued status
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[GhCheckSuite {
                    status: GhCheckStatus::Queued,
                    conclusion: None,
                    ..base_suite.clone()
                }],
                exclude_check_suite_ids: &[],
                wait_for_initial_checks: false
            }
            .run(),
            ChecksStatus::Waiting
        );

        // Suite should be skipped
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[GhCheckSuite {
                    id: 1,
                    status: GhCheckStatus::Queued,
                    conclusion: None,
                    ..base_suite.clone()
                }],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[1]
            }
            .run(),
            ChecksStatus::Skipped
        );

        // Ignore unsupported apps
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[GhCheckSuite {
                    status: GhCheckStatus::Queued,
                    app: GhApplication {
                        slug: "toto".into(),
                        ..GhApplication::default()
                    },
                    ..GhCheckSuite::default()
                }],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Skipped
        );

        // Success
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[GhCheckSuite {
                    status: GhCheckStatus::Completed,
                    conclusion: Some(GhCheckConclusion::Success),
                    ..base_suite.clone()
                }],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Pass
        );

        // Success with skipped
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 2,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_suite.clone()
                    }
                ],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Pass
        );

        // Success with queued
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 2,
                        status: GhCheckStatus::Queued,
                        conclusion: None,
                        ..base_suite.clone()
                    }
                ],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Waiting
        );

        // One failing check make the status fail
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 2,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        ..base_suite.clone()
                    }
                ],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Fail
        );

        // Two GitHub actions at different moments
        let now = OffsetDateTime::now_utc();
        assert_eq!(
            FilterCheckSuitesUseCase {
                check_suites: &[
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Success),
                        updated_at: now + Duration::hours(1),
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 1,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Failure),
                        updated_at: now,
                        ..base_suite.clone()
                    },
                    GhCheckSuite {
                        id: 2,
                        status: GhCheckStatus::Completed,
                        conclusion: Some(GhCheckConclusion::Skipped),
                        ..base_suite
                    }
                ],
                wait_for_initial_checks: false,
                exclude_check_suite_ids: &[]
            }
            .run(),
            ChecksStatus::Pass
        );
    }
}
