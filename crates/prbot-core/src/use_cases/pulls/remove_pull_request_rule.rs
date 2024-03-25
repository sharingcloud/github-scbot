use async_trait::async_trait;
use prbot_models::Repository;
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait RemovePullRequestRuleInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &Repository,
        rule_name: &str,
    ) -> Result<bool>;
}

#[derive(Component)]
#[shaku(interface = RemovePullRequestRuleInterface)]
pub(crate) struct RemovePullRequestRule;

#[async_trait]
impl RemovePullRequestRuleInterface for RemovePullRequestRule {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &Repository,
        rule_name: &str,
    ) -> Result<bool> {
        let status = ctx
            .db_service
            .pull_request_rules_delete(&repository.owner, &repository.name, rule_name)
            .await?;
        Ok(status)
    }
}
