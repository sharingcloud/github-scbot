//! Database pull request models

use diesel::prelude::*;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use super::super::schema::pull_request::{self, dsl};
use super::repository::RepositoryModel;
use super::DbConn;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    None,
    Waiting,
    Pass,
    Fail,
}

impl CheckStatus {
    pub fn from_str(value: &str) -> Result<Self> {
        Ok(match value {
            "none" => Self::None,
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
            Self::None => "none",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[table_name = "pull_request"]
pub struct PullRequestModel {
    pub id: i32,
    pub repository_id: i32,
    pub number: i32,
    pub name: String,
    pub automerge: bool,
    pub check_status: String,
    pub step: String,
    pub status_comment_id: i32,
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
    pub fn check_status_enum(&self) -> Result<CheckStatus> {
        CheckStatus::from_str(&self.check_status)
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

    pub fn update(conn: &DbConn, entry: &Self) -> Result<()> {
        diesel::update(dsl::pull_request)
            .filter(dsl::id.eq(entry.id))
            .set(entry)
            .execute(conn)?;

        Ok(())
    }

    pub fn get_repository_model(conn: &DbConn, entry: &Self) -> Result<RepositoryModel> {
        RepositoryModel::get_from_id(conn, entry.repository_id)
            .ok_or_else(|| eyre!("Missing repository."))
    }
}
