//! Database pull request models

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::DbConn;
use super::{repository::RepositoryModel, ReviewModel};
use crate::database::errors::{DatabaseError, Result};
use crate::database::schema::pull_request::{self, dsl};
use crate::database::schema::repository;

#[derive(Debug, Copy, Clone)]
pub enum StepLabel {
    Wip,
    AwaitingChecks,
    AwaitingChecksChanges,
    AwaitingReview,
    AwaitingReviewChanges,
    AwaitingQA,
    AwaitingMerge,
}

impl StepLabel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wip => "step/wip",
            Self::AwaitingChecks => "step/awaiting-checks",
            Self::AwaitingChecksChanges => "step/awaiting-checks-changes",
            Self::AwaitingReview => "step/awaiting-review",
            Self::AwaitingReviewChanges => "step/awaiting-review-changes",
            Self::AwaitingQA => "step/awaiting-qa",
            Self::AwaitingMerge => "step/awaiting-merge",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "step/wip" => Self::Wip,
            "step/awaiting-checks" => Self::AwaitingChecks,
            "step/awaiting-checks-changes" => Self::AwaitingChecksChanges,
            "step/awaiting-review" => Self::AwaitingReview,
            "step/awaiting-review-changes" => Self::AwaitingReviewChanges,
            "step/awaiting-qa" => Self::AwaitingQA,
            "step/awaiting-merge" => Self::AwaitingMerge,
            e => return Err(DatabaseError::UnknownStepLabelError(e.to_string())),
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Waiting,
    Skipped,
    Pass,
    Fail,
}

impl CheckStatus {
    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "pass" => Self::Pass,
            "waiting" => Self::Waiting,
            "skipped" => Self::Skipped,
            "fail" => Self::Fail,
            e => return Err(DatabaseError::UnknownCheckStatusError(e.to_string())),
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Skipped => "skipped",
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QAStatus {
    Waiting,
    Skipped,
    Pass,
    Fail,
}

impl QAStatus {
    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "pass" => Self::Pass,
            "waiting" => Self::Waiting,
            "fail" => Self::Fail,
            "skipped" => Self::Skipped,
            e => return Err(DatabaseError::UnknownQAStatusError(e.to_string())),
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Pass => "pass",
            Self::Skipped => "skipped",
            Self::Fail => "fail",
        }
    }
}

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Insertable,
    Identifiable,
    Clone,
    PartialEq,
    Eq,
    AsChangeset,
)]
#[table_name = "pull_request"]
pub struct PullRequestModel {
    pub id: i32,
    pub repository_id: i32,
    pub number: i32,
    pub name: String,
    pub automerge: bool,
    pub step: Option<String>,
    pub check_status: Option<String>,
    pub status_comment_id: i32,
    pub qa_status: Option<String>,
    pub wip: bool,
    pub needed_reviewers_count: i32,
}

impl Default for PullRequestModel {
    fn default() -> Self {
        Self {
            id: 0,
            repository_id: 0,
            number: 0,
            name: String::new(),
            automerge: false,
            step: None,
            check_status: None,
            status_comment_id: 0,
            qa_status: None,
            wip: false,
            needed_reviewers_count: 2,
        }
    }
}

#[derive(Default, Insertable)]
#[table_name = "pull_request"]
pub struct PullRequestCreation<'a> {
    pub repository_id: i32,
    pub number: i32,
    pub name: &'a str,
    pub automerge: bool,
    pub check_status: Option<&'a str>,
    pub step: Option<&'a str>,
}

impl PullRequestModel {
    pub fn check_status_enum(&self) -> Option<CheckStatus> {
        self.check_status
            .as_ref()
            .and_then(|x| CheckStatus::from_str(x).ok())
    }

    pub fn qa_status_enum(&self) -> Option<QAStatus> {
        self.qa_status
            .as_ref()
            .and_then(|x| QAStatus::from_str(x).ok())
    }

    pub fn step_enum(&self) -> Option<StepLabel> {
        self.step.as_ref().and_then(|x| StepLabel::from_str(x).ok())
    }

    pub fn update_from_instance(&mut self, conn: &DbConn, other: &Self) -> Result<()> {
        self.name = other.name.clone();
        self.automerge = other.automerge;
        self.step = other.step.clone();
        self.check_status = other.check_status.clone();
        self.status_comment_id = other.status_comment_id;
        self.qa_status = other.qa_status.clone();
        self.wip = other.wip;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_step(&mut self, conn: &DbConn, step: Option<StepLabel>) -> Result<()> {
        self.step = step.map(|x| x.as_str().to_string());
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_step_auto(&mut self, conn: &DbConn) -> Result<()> {
        let step = if self.wip {
            Some(StepLabel::Wip)
        } else {
            match self.check_status_enum() {
                Some(CheckStatus::Pass) | Some(CheckStatus::Skipped) | None => {
                    Some(StepLabel::AwaitingReview)
                }
                Some(CheckStatus::Waiting) => Some(StepLabel::AwaitingChecks),
                Some(CheckStatus::Fail) => Some(StepLabel::AwaitingChecksChanges),
            }
        };

        self.update_step(conn, step)
    }

    pub fn update_check_status(
        &mut self,
        conn: &DbConn,
        check_status: Option<CheckStatus>,
    ) -> Result<()> {
        self.check_status = check_status.map(|x| x.as_str().to_string());
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_wip(&mut self, conn: &DbConn, wip: bool) -> Result<()> {
        self.wip = wip;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_name(&mut self, conn: &DbConn, name: &str) -> Result<()> {
        self.name = name.to_string();
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn update_status_comment(&mut self, conn: &DbConn, status_comment_id: u64) -> Result<()> {
        self.status_comment_id = status_comment_id as i32;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_automerge(&mut self, conn: &DbConn, automerge: bool) -> Result<()> {
        self.automerge = automerge;
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_qa_status(&mut self, conn: &DbConn, status: Option<QAStatus>) -> Result<()> {
        self.qa_status = status.map(|x| x.as_str().to_string());
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn get_checks_url(&self, repo: &RepositoryModel) -> String {
        return format!(
            "https://github.com/{}/{}/pull/{}/checks",
            repo.owner, repo.name, self.number
        );
    }

    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::pull_request.load::<Self>(conn).map_err(Into::into)
    }

    pub fn get_from_number(conn: &DbConn, repo_id: i32, pr_number: i32) -> Option<Self> {
        dsl::pull_request
            .filter(dsl::repository_id.eq(repo_id))
            .filter(dsl::number.eq(pr_number))
            .first(conn)
            .ok()
    }

    pub fn get_from_path_and_number(
        conn: &DbConn,
        path: &str,
        pr_number: i32,
    ) -> Result<Option<(Self, Option<RepositoryModel>)>> {
        let (owner, name) = RepositoryModel::extract_name_from_path(path)?;

        Ok(pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .filter(pull_request::id.eq(pr_number))
            .first(conn)
            .ok())
    }

    pub fn list_from_path(
        conn: &DbConn,
        path: &str,
    ) -> Result<Vec<(Self, Option<RepositoryModel>)>> {
        let (owner, name) = RepositoryModel::extract_name_from_path(path)?;

        pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .get_results(conn)
            .map_err(Into::into)
    }

    #[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
    pub fn create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        diesel::insert_into(dsl::pull_request)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_number(conn, entry.repository_id, entry.number).ok_or_else(|| {
            DatabaseError::UnknownPullRequestError(
                entry.number as u64,
                entry.repository_id.to_string(),
            )
        })
    }

    pub fn get_or_create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        Self::get_from_number(conn, entry.repository_id, entry.number)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }

    pub fn get_repository_model(conn: &DbConn, entry: &Self) -> Result<RepositoryModel> {
        RepositoryModel::get_from_id(conn, entry.repository_id)
            .ok_or_else(|| DatabaseError::UnknownRepositoryError(entry.repository_id.to_string()))
    }

    pub fn get_reviews(&self, conn: &DbConn) -> Result<Vec<ReviewModel>> {
        ReviewModel::list_for_pull_request_id(conn, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::{PullRequestCreation, PullRequestModel};
    use crate::{
        database::{
            establish_single_connection,
            models::{RepositoryCreation, RepositoryModel},
        },
        utils::test_init,
    };

    #[test]
    fn create_pull_request() {
        test_init();

        let conn = establish_single_connection().unwrap();
        let repo = RepositoryModel::create(
            &conn,
            RepositoryCreation {
                name: "TestRepo",
                owner: "me",
            },
        )
        .unwrap();

        let pr = PullRequestModel::create(
            &conn,
            PullRequestCreation {
                repository_id: repo.id,
                number: 1234,
                name: "Toto",
                automerge: false,
                check_status: None,
                step: None,
            },
        )
        .unwrap();

        assert_eq!(pr.id, 1);
        assert_eq!(pr.repository_id, repo.id);
        assert_eq!(pr.number, 1234);
    }
}
