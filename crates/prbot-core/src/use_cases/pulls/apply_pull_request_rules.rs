use async_trait::async_trait;
use prbot_models::{PullRequestHandle, PullRequestRule, QaStatus, RuleAction};
use shaku::{Component, Interface};
use tracing::info;

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait ApplyPullRequestRulesInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        rules: Vec<PullRequestRule>,
    ) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = ApplyPullRequestRulesInterface)]
pub(crate) struct ApplyPullRequestRules;

#[async_trait]
impl ApplyPullRequestRulesInterface for ApplyPullRequestRules {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        pr_handle: &PullRequestHandle,
        rules: Vec<PullRequestRule>,
    ) -> Result<()> {
        for rule in rules {
            for action in rule.actions {
                self.apply_action(ctx, pr_handle, action).await?;
            }
        }

        Ok(())
    }
}

impl ApplyPullRequestRules {
    async fn apply_action(
        &self,
        ctx: &CoreContext<'_>,
        pr_handle: &PullRequestHandle,
        action: RuleAction,
    ) -> Result<()> {
        info!(
            action = ?action,
            pr_handle = ?pr_handle,
            "Applying rule action"
        );

        match action {
            RuleAction::SetAutomerge(value) => {
                ctx.db_service
                    .pull_requests_set_automerge(
                        pr_handle.owner(),
                        pr_handle.name(),
                        pr_handle.number(),
                        value,
                    )
                    .await?;
            }
            RuleAction::SetChecksEnabled(value) => {
                ctx.db_service
                    .pull_requests_set_checks_enabled(
                        pr_handle.owner(),
                        pr_handle.name(),
                        pr_handle.number(),
                        value,
                    )
                    .await?;
            }
            RuleAction::SetNeededReviewers(value) => {
                ctx.db_service
                    .pull_requests_set_needed_reviewers_count(
                        pr_handle.owner(),
                        pr_handle.name(),
                        pr_handle.number(),
                        value,
                    )
                    .await?;
            }
            RuleAction::SetQaEnabled(value) => {
                ctx.db_service
                    .pull_requests_set_qa_status(
                        pr_handle.owner(),
                        pr_handle.name(),
                        pr_handle.number(),
                        if value {
                            QaStatus::Waiting
                        } else {
                            QaStatus::Skipped
                        },
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_models::{PullRequest, PullRequestRule, Repository, RuleAction, RuleCondition};

    use super::{ApplyPullRequestRules, ApplyPullRequestRulesInterface};
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn apply() {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let pr = ctx
            .db_service
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                automerge: false,
                ..Default::default()
            })
            .await
            .unwrap();

        assert!(!pr.automerge);

        let rule1 = ctx
            .db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 1".into(),
                actions: vec![RuleAction::SetAutomerge(true)],
                conditions: vec![RuleCondition::Author("me".into())],
            })
            .await
            .unwrap();

        let handle = ("me", "name", pr.number).into();

        ApplyPullRequestRules
            .run(&ctx.as_context(), &handle, vec![rule1])
            .await
            .unwrap();

        let pr = ctx
            .db_service
            .pull_requests_get("me", "name", pr.number)
            .await
            .unwrap()
            .unwrap();

        assert!(pr.automerge);
    }
}
