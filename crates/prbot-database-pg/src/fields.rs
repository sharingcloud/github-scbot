use std::ops::Deref;

use prbot_models::{MergeStrategy, QaStatus, RuleAction, RuleBranch, RuleCondition};
use sqlx::{
    postgres::{PgTypeInfo, PgValueRef},
    Decode, Postgres, Type,
};

pub struct RuleBranchDecode(RuleBranch);
impl<'r> Decode<'r, Postgres> for RuleBranchDecode {
    fn decode(value: PgValueRef) -> core::result::Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        Ok(Self(RuleBranch::from(str_value)))
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

pub struct MergeStrategyDecode(MergeStrategy);
impl<'r> Decode<'r, Postgres> for MergeStrategyDecode {
    fn decode(value: PgValueRef) -> core::result::Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        MergeStrategy::try_from(str_value)
            .map(Self)
            .map_err(Into::into)
    }
}

impl Type<Postgres> for MergeStrategyDecode {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("varchar")
    }
}

impl Deref for MergeStrategyDecode {
    type Target = MergeStrategy;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MergeStrategyDecode> for MergeStrategy {
    fn from(v: MergeStrategyDecode) -> Self {
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

pub struct RuleActionsDecode(Vec<RuleAction>);
impl<'r> Decode<'r, Postgres> for RuleActionsDecode {
    fn decode(value: PgValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        let json_value: Vec<RuleAction> = serde_json::from_str(str_value).unwrap();
        Ok(Self(json_value))
    }
}

impl Type<Postgres> for RuleActionsDecode {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl Deref for RuleActionsDecode {
    type Target = Vec<RuleAction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RuleConditionsDecode(Vec<RuleCondition>);
impl<'r> Decode<'r, Postgres> for RuleConditionsDecode {
    fn decode(value: PgValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let str_value = <&str as Decode<Postgres>>::decode(value)?;
        let json_value: Vec<RuleCondition> = serde_json::from_str(str_value).unwrap();
        Ok(Self(json_value))
    }
}

impl Type<Postgres> for RuleConditionsDecode {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl Deref for RuleConditionsDecode {
    type Target = Vec<RuleCondition>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
