use async_trait::async_trait;
use prbot_models::{PullRequestRule, Repository, RuleAction, RuleCondition};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait AddPullRequestRuleInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &Repository,
        name: String,
        conditions: Vec<RuleCondition>,
        actions: Vec<RuleAction>,
    ) -> Result<PullRequestRule>;
}

#[derive(Component)]
#[shaku(interface = AddPullRequestRuleInterface)]
pub(crate) struct AddPullRequestRule;

#[async_trait]
impl AddPullRequestRuleInterface for AddPullRequestRule {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository: &Repository,
        name: String,
        conditions: Vec<RuleCondition>,
        actions: Vec<RuleAction>,
    ) -> Result<PullRequestRule> {
        let rule = PullRequestRule {
            repository_id: repository.id,
            name,
            conditions,
            actions,
        };

        let rule = ctx.db_service.pull_request_rules_create(rule).await?;
        Ok(rule)
    }
}
