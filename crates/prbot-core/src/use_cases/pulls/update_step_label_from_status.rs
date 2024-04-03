use async_trait::async_trait;
use prbot_models::{PullRequestHandle, StepLabel};
use shaku::{Component, HasComponent, Interface};

use super::SetStepLabelInterface;
use crate::{
    use_cases::status::{PullRequestStatus, StepLabelChooser},
    CoreContext, Result,
};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait UpdateStepLabelFromStatusInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
    ) -> Result<StepLabel>;
}

#[derive(Component)]
#[shaku(interface = UpdateStepLabelFromStatusInterface)]
pub(crate) struct UpdateStepLabelFromStatus;

#[async_trait]
impl UpdateStepLabelFromStatusInterface for UpdateStepLabelFromStatus {
    #[tracing::instrument(skip(self, ctx), fields(pr_handle, label))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        pr_status: &PullRequestStatus,
    ) -> Result<StepLabel> {
        let step_label = StepLabelChooser::default().choose_from_status(pr_status);
        let set_step_label: &dyn SetStepLabelInterface = ctx.core_module.resolve_ref();
        set_step_label.run(ctx, pr_handle, Some(step_label)).await?;
        Ok(step_label)
    }
}
