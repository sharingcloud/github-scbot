use prbot_models::{
    Account, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, PullRequestRule,
    Repository, RequiredReviewer,
};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::fields::{
    MergeStrategyDecode, QaStatusDecode, RuleActionsDecode, RuleBranchDecode, RuleConditionsDecode,
};

pub(crate) struct AccountRow(Account);
pub(crate) struct ExternalAccountRow(ExternalAccount);
pub(crate) struct ExternalAccountRightRow(ExternalAccountRight);
pub(crate) struct MergeRuleRow(MergeRule);
pub(crate) struct PullRequestRow(PullRequest);
pub(crate) struct RepositoryRow(Repository);
pub(crate) struct RequiredReviewerRow(RequiredReviewer);
pub(crate) struct PullRequestRuleRow(PullRequestRule);

impl From<AccountRow> for Account {
    fn from(r: AccountRow) -> Self {
        r.0
    }
}

impl From<ExternalAccountRow> for ExternalAccount {
    fn from(r: ExternalAccountRow) -> Self {
        r.0
    }
}

impl From<ExternalAccountRightRow> for ExternalAccountRight {
    fn from(r: ExternalAccountRightRow) -> Self {
        r.0
    }
}

impl From<MergeRuleRow> for MergeRule {
    fn from(r: MergeRuleRow) -> Self {
        r.0
    }
}

impl From<PullRequestRow> for PullRequest {
    fn from(r: PullRequestRow) -> Self {
        r.0
    }
}

impl From<RepositoryRow> for Repository {
    fn from(r: RepositoryRow) -> Self {
        r.0
    }
}

impl From<RequiredReviewerRow> for RequiredReviewer {
    fn from(r: RequiredReviewerRow) -> Self {
        r.0
    }
}

impl From<PullRequestRuleRow> for PullRequestRule {
    fn from(r: PullRequestRuleRow) -> Self {
        r.0
    }
}

impl<'r> FromRow<'r, PgRow> for AccountRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(Account {
            username: row.try_get("username")?,
            is_admin: row.try_get("is_admin")?,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccountRightRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(ExternalAccountRight {
            username: row.try_get("username")?,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for ExternalAccountRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(ExternalAccount {
            username: row.try_get("username")?,
            public_key: row.try_get("public_key")?,
            private_key: row.try_get("private_key")?,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for MergeRuleRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(MergeRule {
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            base_branch: row.try_get::<RuleBranchDecode, _>("base_branch")?.clone(),
            head_branch: row.try_get::<RuleBranchDecode, _>("head_branch")?.clone(),
            strategy: *row.try_get::<MergeStrategyDecode, _>("strategy")?,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for PullRequestRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(PullRequest {
            id: row.try_get::<i32, _>("id")? as u64,
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            number: row.try_get::<i32, _>("number")? as u64,
            qa_status: *row.try_get::<QaStatusDecode, _>("qa_status")?,
            needed_reviewers_count: row.try_get::<i32, _>("needed_reviewers_count")? as u64,
            status_comment_id: row.try_get::<i32, _>("status_comment_id")? as u64,
            checks_enabled: row.try_get("checks_enabled")?,
            automerge: row.try_get("automerge")?,
            locked: row.try_get("locked")?,
            strategy_override: row
                .try_get::<Option<MergeStrategyDecode>, _>("strategy_override")?
                .map(Into::into),
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for RepositoryRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(Repository {
            id: row.try_get::<i32, _>("id")? as u64,
            owner: row.try_get("owner")?,
            name: row.try_get("name")?,
            manual_interaction: row.try_get("manual_interaction")?,
            pr_title_validation_regex: row.try_get("pr_title_validation_regex")?,
            default_strategy: *row.try_get::<MergeStrategyDecode, _>("default_strategy")?,
            default_needed_reviewers_count: row
                .try_get::<i32, _>("default_needed_reviewers_count")?
                as u64,
            default_automerge: row.try_get("default_automerge")?,
            default_enable_qa: row.try_get("default_enable_qa")?,
            default_enable_checks: row.try_get("default_enable_checks")?,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for RequiredReviewerRow {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self(RequiredReviewer {
            pull_request_id: row.try_get::<i32, _>("pull_request_id")? as u64,
            username: row.try_get("username")?,
        }))
    }
}

impl<'r> FromRow<'r, PgRow> for PullRequestRuleRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self(PullRequestRule {
            repository_id: row.try_get::<i32, _>("repository_id")? as u64,
            name: row.try_get("name")?,
            conditions: (*row.try_get::<RuleConditionsDecode, _>("conditions")?).clone(),
            actions: (*row.try_get::<RuleActionsDecode, _>("actions")?).clone(),
        }))
    }
}
