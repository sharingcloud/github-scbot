use github_scbot_core::types::{pulls::GhMergeStrategy, status::QaStatus};
use github_scbot_macros::SCGetter;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{
    fields::{GhMergeStrategyDecode, QaStatusDecode},
    Repository,
};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder, Serialize, Deserialize)]
#[builder(default, setter(into))]
pub struct PullRequest {
    #[get]
    pub(crate) id: u64,
    #[get]
    pub(crate) repository_id: u64,
    #[get]
    pub(crate) number: u64,
    #[get_ref]
    pub(crate) qa_status: QaStatus,
    #[get]
    pub(crate) needed_reviewers_count: u64,
    #[get]
    pub(crate) status_comment_id: u64,
    #[get]
    pub(crate) checks_enabled: bool,
    #[get]
    pub(crate) automerge: bool,
    #[get]
    pub(crate) locked: bool,
    #[get_ref]
    pub(crate) strategy_override: Option<GhMergeStrategy>,
}

impl PullRequest {
    pub fn builder() -> PullRequestBuilder {
        PullRequestBuilder::default()
    }

    pub fn set_repository_id(&mut self, id: u64) {
        self.repository_id = id;
    }
}

impl PullRequestBuilder {
    pub fn with_repository(&mut self, repository: &Repository) -> &mut Self {
        self.repository_id = Some(repository.id());
        self.automerge = Some(repository.default_automerge());
        self.checks_enabled = Some(repository.default_enable_checks());
        self.needed_reviewers_count = Some(repository.default_needed_reviewers_count());
        self.qa_status = if repository.default_enable_qa() {
            None
        } else {
            Some(QaStatus::Skipped)
        };
        self
    }
}

impl<'r> FromRow<'r, PgRow> for PullRequest {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
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
                .try_get::<Option<GhMergeStrategyDecode>, _>("strategy_override")?
                .map(Into::into),
        })
    }
}
