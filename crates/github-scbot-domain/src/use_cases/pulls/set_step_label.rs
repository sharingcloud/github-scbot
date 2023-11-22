use async_trait::async_trait;
use github_scbot_domain_models::{PullRequestHandle, StepLabel};
use github_scbot_ghapi_interface::ApiService;

use crate::Result;

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait(?Send)]
pub trait SetStepLabelUseCaseInterface {
    async fn run(&self, pr_handle: &PullRequestHandle, label: Option<StepLabel>) -> Result<()>;
}

pub struct SetStepLabelUseCase<'a> {
    pub api_service: &'a dyn ApiService,
}

#[async_trait(?Send)]
impl<'a> SetStepLabelUseCaseInterface for SetStepLabelUseCase<'a> {
    #[tracing::instrument(skip(self), fields(pr_handle, label))]
    async fn run(&self, pr_handle: &PullRequestHandle, label: Option<StepLabel>) -> Result<()> {
        let previous_labels = self
            .api_service
            .issue_labels_list(
                pr_handle.repository().owner(),
                pr_handle.repository().name(),
                pr_handle.number(),
            )
            .await?;
        let new_labels = Self::add_step_in_existing_labels(&previous_labels, label);

        if previous_labels != new_labels {
            self.api_service
                .issue_labels_replace_all(
                    pr_handle.repository().owner(),
                    pr_handle.repository().name(),
                    pr_handle.number(),
                    &new_labels,
                )
                .await?;
        }

        Ok(())
    }
}

impl<'a> SetStepLabelUseCase<'a> {
    /// Add pull request step label in existing labels by returning a new vector.
    pub fn add_step_in_existing_labels(
        existing_labels: &[String],
        step: Option<StepLabel>,
    ) -> Vec<String> {
        let mut preserved_labels: Vec<String> = existing_labels
            .iter()
            .filter(|&x| StepLabel::try_from(&x[..]).is_err())
            .cloned()
            .collect();

        if let Some(step) = step {
            preserved_labels.push(step.to_str().to_string());
        }

        preserved_labels
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[test]
    fn test_add_step_in_existing_labels() {
        // A step label remove existing step labels.
        assert_eq!(
            SetStepLabelUseCase::add_step_in_existing_labels(
                &[
                    "label1".into(),
                    "label2".into(),
                    StepLabel::AwaitingMerge.to_str().into()
                ],
                Some(StepLabel::AwaitingQa)
            ),
            vec![
                "label1".to_string(),
                "label2".to_string(),
                StepLabel::AwaitingQa.to_str().into()
            ]
        );

        // No step label remove existing step labels.
        assert_eq!(
            SetStepLabelUseCase::add_step_in_existing_labels(
                &[
                    "label1".into(),
                    "label2".into(),
                    StepLabel::AwaitingMerge.to_str().into()
                ],
                None
            ),
            vec!["label1".to_string(), "label2".to_string()]
        )
    }

    #[tokio::test]
    async fn remove_step_labels() -> Result<()> {
        let mut adapter = MockApiService::new();
        adapter
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, pr_number| owner == "owner" && name == "name" && pr_number == &1)
            .return_once(|_, _, _| {
                Ok(vec![
                    "dummy".into(),
                    "step/awaiting-changes".into(),
                    "step/awaiting-review".into(),
                ])
            });

        adapter
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, pr_number, labels| {
                owner == "owner"
                    && name == "name"
                    && pr_number == &1
                    && labels == ["dummy".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        SetStepLabelUseCase {
            api_service: &adapter,
        }
        .run(&("owner", "name", 1).into(), None)
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn replace_step_label_with_another() -> Result<()> {
        let mut adapter = MockApiService::new();
        adapter
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, pr_number| owner == "owner" && name == "name" && pr_number == &1)
            .return_once(|_, _, _| {
                Ok(vec![
                    "dummy".into(),
                    "step/awaiting-changes".into(),
                    "step/awaiting-review".into(),
                ])
            });

        adapter
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, pr_number, labels| {
                owner == "owner"
                    && name == "name"
                    && pr_number == &1
                    && labels == ["dummy".to_string(), "step/wip".into()]
            })
            .return_once(|_, _, _, _| Ok(()));

        SetStepLabelUseCase {
            api_service: &adapter,
        }
        .run(&("owner", "name", 1).into(), Some(StepLabel::Wip))
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn no_need_to_replace_labels() -> Result<()> {
        let mut api_service = MockApiService::new();
        api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, pr_number| owner == "owner" && name == "name" && pr_number == &1)
            .return_once(|_, _, _| Ok(vec!["dummy".into(), "step/awaiting-review".into()]));

        SetStepLabelUseCase {
            api_service: &api_service,
        }
        .run(
            &("owner", "name", 1).into(),
            Some(StepLabel::AwaitingReview),
        )
        .await?;

        Ok(())
    }
}
