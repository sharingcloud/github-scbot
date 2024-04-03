use async_trait::async_trait;
use prbot_ghapi_interface::types::GhPullRequest;
use prbot_models::{PullRequestRule, RepositoryPath, RuleBranch, RuleCondition};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait ResolvePullRequestRulesInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        upstream_pr: &GhPullRequest,
    ) -> Result<Vec<PullRequestRule>>;
}

#[derive(Component)]
#[shaku(interface = ResolvePullRequestRulesInterface)]
pub(crate) struct ResolvePullRequestRules;

#[async_trait]
impl ResolvePullRequestRulesInterface for ResolvePullRequestRules {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
        upstream_pr: &GhPullRequest,
    ) -> Result<Vec<PullRequestRule>> {
        fn match_condition(condition: &RuleCondition, upstream_pr: &GhPullRequest) -> bool {
            match condition {
                RuleCondition::Author(author) => upstream_pr.user.login == *author,
                RuleCondition::BaseBranch(branch) => match branch {
                    RuleBranch::Named(branch) => upstream_pr.base.reference == *branch,
                    RuleBranch::Wildcard => true,
                },
                RuleCondition::HeadBranch(branch) => match branch {
                    RuleBranch::Named(branch) => upstream_pr.head.reference == *branch,
                    RuleBranch::Wildcard => true,
                },
            }
        }

        fn match_all_conditions(conditions: &[RuleCondition], upstream_pr: &GhPullRequest) -> bool {
            conditions.iter().all(|c| match_condition(c, upstream_pr))
        }

        Ok(ctx
            .db_service
            .pull_request_rules_list(repository_path.owner(), repository_path.name())
            .await?
            .into_iter()
            .filter(|r| !r.conditions.is_empty() && !r.actions.is_empty())
            .filter(|r| match_all_conditions(&r.conditions, upstream_pr))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_ghapi_interface::types::{GhPullRequest, GhUser};
    use prbot_models::{PullRequestRule, Repository, RuleAction, RuleCondition};

    use super::{ResolvePullRequestRules, ResolvePullRequestRulesInterface};
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn resolve_empty_no_match() {
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

        ctx.db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 1".into(),
                actions: vec![],
                conditions: vec![],
            })
            .await
            .unwrap();

        let upstream_pr = GhPullRequest {
            ..Default::default()
        };

        let rules = ResolvePullRequestRules
            .run(&ctx.as_context(), &repo.path(), &upstream_pr)
            .await
            .unwrap();

        assert_eq!(rules, vec![]);
    }

    #[tokio::test]
    async fn resolve_match() {
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

        let upstream_pr = GhPullRequest {
            user: GhUser {
                login: "bot".into(),
            },
            ..Default::default()
        };

        let rule1 = ctx
            .db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 1".into(),
                actions: vec![RuleAction::SetAutomerge(true)],
                conditions: vec![RuleCondition::Author("bot".into())],
            })
            .await
            .unwrap();

        let rules = ResolvePullRequestRules
            .run(&ctx.as_context(), &repo.path(), &upstream_pr)
            .await
            .unwrap();

        assert_eq!(rules, vec![rule1]);
    }

    #[tokio::test]
    async fn resolve_two_matches() {
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

        let upstream_pr = GhPullRequest {
            user: GhUser {
                login: "bot".into(),
            },
            ..Default::default()
        };

        let rule1 = ctx
            .db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 1".into(),
                actions: vec![RuleAction::SetAutomerge(true)],
                conditions: vec![RuleCondition::Author("bot".into())],
            })
            .await
            .unwrap();

        let rule2 = ctx
            .db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 2".into(),
                actions: vec![RuleAction::SetChecksEnabled(true)],
                conditions: vec![RuleCondition::Author("bot".into())],
            })
            .await
            .unwrap();

        ctx.db_service
            .pull_request_rules_create(PullRequestRule {
                repository_id: repo.id,
                name: "Rule 3".into(),
                actions: vec![RuleAction::SetChecksEnabled(true)],
                conditions: vec![RuleCondition::Author("bot1".into())],
            })
            .await
            .unwrap();

        let rules = ResolvePullRequestRules
            .run(&ctx.as_context(), &repo.path(), &upstream_pr)
            .await
            .unwrap();

        assert_eq!(rules, vec![rule1, rule2]);
    }
}
