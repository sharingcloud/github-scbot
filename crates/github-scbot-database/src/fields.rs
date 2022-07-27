use std::ops::Deref;

use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch, status::QaStatus};
use sqlx::{
    postgres::{PgTypeInfo, PgValueRef},
    Decode, Postgres, Type,
};

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
