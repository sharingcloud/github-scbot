use std::ops::Deref;

use github_scbot_types::{pulls::GhMergeStrategy, status::QaStatus};
use sqlx::{
    postgres::{PgTypeInfo, PgValueRef},
    Decode, Postgres, Type,
};

/// Rule branch.
#[derive(Clone, Debug)]
pub enum RuleBranch {
    /// Named.
    Named(String),
    /// Wildcard.
    Wildcard,
}

impl From<&str> for RuleBranch {
    fn from(value: &str) -> Self {
        match value {
            "*" => Self::Wildcard,
            n => Self::Named(n.into()),
        }
    }
}

impl ToString for RuleBranch {
    fn to_string(&self) -> String {
        self.name()
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

pub struct RuleBranchDecode(RuleBranch);
impl<'r> Decode<'r, Postgres> for RuleBranchDecode {
    fn decode(value: PgValueRef) -> core::result::Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        RuleBranch::try_from(str_value)
            .map(Self)
            .map_err(Into::into)
    }
}

impl Type<Postgres> for RuleBranchDecode {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("varchar")
    }
}

impl Deref for RuleBranchDecode {
    type Target = RuleBranch;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct GhMergeStrategyDecode(GhMergeStrategy);
impl<'r> Decode<'r, Postgres> for GhMergeStrategyDecode {
    fn decode(value: PgValueRef) -> core::result::Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        GhMergeStrategy::try_from(str_value)
            .map(Self)
            .map_err(Into::into)
    }
}

impl Type<Postgres> for GhMergeStrategyDecode {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("varchar")
    }
}

impl Deref for GhMergeStrategyDecode {
    type Target = GhMergeStrategy;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<GhMergeStrategyDecode> for GhMergeStrategy {
    fn from(v: GhMergeStrategyDecode) -> Self {
        v.0
    }
}

pub struct QaStatusDecode(QaStatus);
impl<'r> Decode<'r, Postgres> for QaStatusDecode {
    fn decode(value: PgValueRef) -> core::result::Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        QaStatus::try_from(str_value).map(Self).map_err(Into::into)
    }
}

impl Type<Postgres> for QaStatusDecode {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("varchar")
    }
}

impl Deref for QaStatusDecode {
    type Target = QaStatus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
