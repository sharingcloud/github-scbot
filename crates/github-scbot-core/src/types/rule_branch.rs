//! Rule branch.

use std::{convert::Infallible, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

/// Rule branch.
#[derive(Clone, Debug, PartialEq)]
pub enum RuleBranch {
    /// Named.
    Named(String),
    /// Wildcard.
    Wildcard,
}

impl Serialize for RuleBranch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name())
    }
}

struct RuleBranchVisitor;

impl<'de> Visitor<'de> for RuleBranchVisitor {
    type Value = RuleBranch;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid branch name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }
}

impl<'de> Deserialize<'de> for RuleBranch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(RuleBranchVisitor)
    }
}

impl From<&str> for RuleBranch {
    fn from(value: &str) -> Self {
        match value {
            "*" => Self::Wildcard,
            n => Self::Named(n.into()),
        }
    }
}

impl FromStr for RuleBranch {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl RuleBranch {
    /// Get branch name.
    pub fn name(&self) -> String {
        match self {
            RuleBranch::Named(s) => s.clone(),
            RuleBranch::Wildcard => "*".into(),
        }
    }
}

impl Default for RuleBranch {
    fn default() -> Self {
        RuleBranch::Wildcard
    }
}

impl std::fmt::Display for RuleBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}
