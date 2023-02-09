use github_scbot_core::{
    time::OffsetDateTime,
    types::{common::GhUser, reviews::GhReviewState},
};
use heck::ToSnakeCase;
use serde::{Deserialize, Serialize};

/// Review state (API version)
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GhReviewStateApi {
    /// Approved.
    Approved,
    /// Changes requested.
    ChangesRequested,
    /// Commented.
    Commented,
    /// Dismissed.
    Dismissed,
    /// Pending.
    Pending,
}

impl From<GhReviewStateApi> for GhReviewState {
    fn from(state_api: GhReviewStateApi) -> Self {
        let str_value = serde_plain::to_string(&state_api).unwrap();
        let snake_case_value = str_value.to_snake_case();
        serde_plain::from_str(&snake_case_value).unwrap()
    }
}

/// Review (API version)
#[derive(Deserialize, Clone, Debug)]
pub struct GhReviewApi {
    /// User.
    pub user: GhUser,
    /// Submitted at.
    #[serde(with = "github_scbot_core::time::serde::rfc3339")]
    pub submitted_at: OffsetDateTime,
    /// State.
    pub state: GhReviewStateApi,
}

impl Default for GhReviewApi {
    fn default() -> Self {
        Self {
            user: GhUser::default(),
            submitted_at: OffsetDateTime::now_utc(),
            state: GhReviewStateApi::Pending,
        }
    }
}
