use async_trait::async_trait;
use github_scbot_domain_models::PullRequestHandle;
use github_scbot_ghapi_interface::ApiService;

use super::utils::text_generator::SummaryTextGenerator;
use crate::{use_cases::status::PullRequestStatus, Result};

const SUMMARY_START_MARKER: &str = "<!-- github-scbot:start-summary -->";
const SUMMARY_END_MARKER: &str = "<!-- github-scbot:end-summary -->";

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait UpdatePullRequestSummaryUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, pr_status: &PullRequestStatus)
        -> Result<()>;
}

pub struct UpdatePullRequestSummaryUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

#[async_trait(?Send)]
impl<'a> UpdatePullRequestSummaryUseCaseInterface for UpdatePullRequestSummaryUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle))]
    async fn run(
        &self,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
    ) -> Result<()> {
        let upstream_pr = self
            .api_service
            .pulls_get(pr_handle.owner(), pr_handle.name(), pr_handle.number())
            .await?;

        let summary = SummaryTextGenerator::generate(pr_status)?;
        let updated_body =
            Self::insert_summary_in_body(&summary, &upstream_pr.body.unwrap_or_default());

        self.api_service
            .pulls_update_body(
                pr_handle.owner(),
                pr_handle.name(),
                pr_handle.number(),
                &updated_body,
            )
            .await?;

        Ok(())
    }
}

impl<'a> UpdatePullRequestSummaryUseCase<'a> {
    fn insert_summary_in_body(summary: &str, body: &str) -> String {
        let start_marker = body.find(SUMMARY_START_MARKER).unwrap_or(body.len());
        let (existing, remaining) = body.split_at(start_marker);

        let remaining = if let Some(end_marker) = remaining.find(SUMMARY_END_MARKER) {
            &remaining[end_marker + SUMMARY_END_MARKER.len()..]
        } else {
            remaining
        };

        [
            existing.trim(),
            "", // A small additional space to make sure the summary is not directly below the PR body
            SUMMARY_START_MARKER,
            "<hr />",
            summary.trim(),
            SUMMARY_END_MARKER,
            remaining.trim(),
        ]
        .join("\n")
        .trim_end()
        .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::{types::GhPullRequest, MockApiService};

    use super::*;

    #[tokio::test]
    async fn run() {
        let api_service = {
            let mut svc = MockApiService::new();

            svc.expect_pulls_get()
                .once()
                .withf(|owner, name, number| owner == "me" && name == "test" && number == &1)
                .return_once(|_, _, _| {
                    Ok(GhPullRequest {
                        number: 1,
                        body: Some("abcd! efgh!\n\nhello!".into()),
                        ..Default::default()
                    })
                });

            svc.expect_pulls_update_body()
                .once()
                .withf(|owner, name, number, body| {
                    owner == "me"
                        && name == "test"
                        && number == &1
                        && body.contains(SUMMARY_START_MARKER)
                        && body.contains(SUMMARY_END_MARKER)
                })
                .return_once(|_, _, _, _| Ok(()));

            svc
        };

        UpdatePullRequestSummaryUseCase {
            api_service: &api_service,
        }
        .run(
            &("me", "test", 1).into(),
            &PullRequestStatus {
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn insert_summary_in_body_no_markers() {
        let output = UpdatePullRequestSummaryUseCase::insert_summary_in_body(
            concat!("abcd\n", "efgh"),
            "Sample PR body.",
        );

        assert_eq!(
            output,
            concat!(
                "Sample PR body.\n",
                "\n",
                "<!-- github-scbot:start-summary -->\n",
                "<hr />\n",
                "abcd\n",
                "efgh\n",
                "<!-- github-scbot:end-summary -->"
            )
        );
    }

    #[tokio::test]
    async fn insert_summary_in_body_with_markers() {
        let output = UpdatePullRequestSummaryUseCase::insert_summary_in_body(
            concat!("abcd\n", "efgh"),
            concat!(
                "Sample PR body.\n",
                "abc\n",
                "\n",
                "<!-- github-scbot:start-summary -->\n",
                "<hr />\n",
                "Hey\n",
                "<!-- github-scbot:end-summary -->\n",
                "After"
            ),
        );

        assert_eq!(
            output,
            concat!(
                "Sample PR body.\n",
                "abc\n",
                "\n",
                "<!-- github-scbot:start-summary -->\n",
                "<hr />\n",
                "abcd\n",
                "efgh\n",
                "<!-- github-scbot:end-summary -->\n",
                "After"
            )
        );
    }
}
