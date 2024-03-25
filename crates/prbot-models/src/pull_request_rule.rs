use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::RuleBranch;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestRule {
    pub repository_id: u64,
    pub name: String,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    SetAutomerge(bool),
    SetQaEnabled(bool),
    SetChecksEnabled(bool),
    SetNeededReviewers(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleCondition {
    BaseBranch(RuleBranch),
    HeadBranch(RuleBranch),
    Author(String),
}

impl FromStr for RuleAction {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl FromStr for RuleCondition {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{RuleAction, RuleBranch, RuleCondition};

    #[test]
    fn parse_rule_condition() {
        let rule = RuleCondition::from_str(r#"{"author": "me"}"#).unwrap();
        assert_eq!(rule, RuleCondition::Author("me".into()));

        let rule = RuleCondition::from_str(r#"{"base_branch": "main"}"#).unwrap();
        assert_eq!(
            rule,
            RuleCondition::BaseBranch(RuleBranch::Named("main".into()))
        );

        let rule = RuleCondition::from_str(r#"{"base_branch": "*"}"#).unwrap();
        assert_eq!(rule, RuleCondition::BaseBranch(RuleBranch::Wildcard));
    }

    #[test]
    fn parse_rule_action() {
        let rule = RuleAction::from_str(r#"{"set_automerge": true}"#).unwrap();
        assert_eq!(rule, RuleAction::SetAutomerge(true));

        let rule = RuleAction::from_str(r#"{"set_needed_reviewers": 0}"#).unwrap();
        assert_eq!(rule, RuleAction::SetNeededReviewers(0));
    }
}
