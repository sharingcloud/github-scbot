use async_trait::async_trait;
use prbot_models::{PullRequestHandle, StepLabel};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait SetStepLabelInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        label: Option<StepLabel>,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = SetStepLabelInterface)]
pub(crate) struct SetStepLabel;

#[async_trait]
impl SetStepLabelInterface for SetStepLabel {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle, label))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        label: Option<StepLabel>,
    ) -> Result<()> {
        let previous_labels = ctx
            .api_service
            .issue_labels_list(
                pr_handle.repository_path().owner(),
                pr_handle.repository_path().name(),
                pr_handle.number(),
            )
            .await?;
        let new_labels = Self::add_step_in_existing_labels(&previous_labels, label);

        if previous_labels != new_labels {
            ctx.api_service
                .issue_labels_replace_all(
                    pr_handle.repository_path().owner(),
                    pr_handle.repository_path().name(),
                    pr_handle.number(),
                    &new_labels,
                )
                .await?;
        }

        Ok(())
    }
}

impl SetStepLabel {
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
    use super::*;
    use crate::context::tests::CoreContextTest;

    #[test]
    fn test_add_step_in_existing_labels() {
        // A step label remove existing step labels.
        assert_eq!(
            SetStepLabel::add_step_in_existing_labels(
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
            SetStepLabel::add_step_in_existing_labels(
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
        let mut ctx = CoreContextTest::new();
        ctx.api_service
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

        ctx.api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, pr_number, labels| {
                owner == "owner"
                    && name == "name"
                    && pr_number == &1
                    && labels == ["dummy".to_string()]
            })
            .return_once(|_, _, _, _| Ok(()));

        SetStepLabel
            .run(&ctx.as_context(), &("owner", "name", 1).into(), None)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn replace_step_label_with_another() -> Result<()> {
        let mut ctx = CoreContextTest::new();
        ctx.api_service
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

        ctx.api_service
            .expect_issue_labels_replace_all()
            .once()
            .withf(|owner, name, pr_number, labels| {
                owner == "owner"
                    && name == "name"
                    && pr_number == &1
                    && labels == ["dummy".to_string(), "step/wip".into()]
            })
            .return_once(|_, _, _, _| Ok(()));

        SetStepLabel
            .run(
                &ctx.as_context(),
                &("owner", "name", 1).into(),
                Some(StepLabel::Wip),
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn no_need_to_replace_labels() -> Result<()> {
        let mut ctx = CoreContextTest::new();
        ctx.api_service
            .expect_issue_labels_list()
            .once()
            .withf(|owner, name, pr_number| owner == "owner" && name == "name" && pr_number == &1)
            .return_once(|_, _, _| Ok(vec!["dummy".into(), "step/awaiting-review".into()]));

        SetStepLabel
            .run(
                &ctx.as_context(),
                &("owner", "name", 1).into(),
                Some(StepLabel::AwaitingReview),
            )
            .await?;

        Ok(())
    }
}
