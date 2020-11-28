//! Database pull request models

use std::convert::TryInto;

use diesel::prelude::*;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use super::repository::RepositoryModel;
use super::DbConn;
use crate::api::labels::StepLabel;
use crate::database::schema::pull_request::{self, dsl};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Waiting,
    Pass,
    Fail,
}

impl CheckStatus {
    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "pass" => Self::Pass,
            "waiting" => Self::Waiting,
            "fail" => Self::Fail,
            e => return Err(eyre!("Bad check status: {}", e)),
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QAStatus {
    Waiting,
    Pass,
    Fail,
}

impl QAStatus {
    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "pass" => Self::Pass,
            "waiting" => Self::Waiting,
            "fail" => Self::Fail,
            e => return Err(eyre!("Bad QA status: {}", e)),
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Identifiable, AsChangeset)]
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
}

#[derive(Insertable)]
#[table_name = "pull_request"]
pub struct PullRequestCreation<'a> {
    pub repository_id: i32,
    pub number: i32,
    pub name: &'a str,
    pub automerge: bool,
    pub check_status: &'a str,
    pub step: &'a str,
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

    pub fn update_step(&mut self, conn: &DbConn, step: Option<StepLabel>) -> Result<()> {
        println!("Updating step for PR #{}: {:?}", self.number, step);
        self.step = step.map(|x| x.as_str().to_string());
        self.save_changes::<Self>(conn)?;

        Ok(())
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

    pub fn update_status_comment(&mut self, conn: &DbConn, status_comment_id: u64) -> Result<()> {
        self.status_comment_id = status_comment_id.try_into()?;
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

    pub fn create(conn: &DbConn, entry: &PullRequestCreation) -> Result<Self> {
        diesel::insert_into(dsl::pull_request)
            .values(entry)
            .execute(conn)?;

        Self::get_from_number(conn, entry.repository_id, entry.number)
            .ok_or_else(|| eyre!("Error while fetching created pull request"))
    }

    pub fn get_or_create(conn: &DbConn, entry: &PullRequestCreation) -> Result<Self> {
        Self::get_from_number(conn, entry.repository_id, entry.number)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }

    pub fn get_repository_model(conn: &DbConn, entry: &Self) -> Result<RepositoryModel> {
        RepositoryModel::get_from_id(conn, entry.repository_id)
            .ok_or_else(|| eyre!("Missing repository."))
    }
}
