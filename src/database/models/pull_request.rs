//! Database pull request models

use diesel::prelude::*;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use super::super::schema::pull_request::{self, dsl};
use super::DbConn;

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name = "pull_request"]
pub struct PullRequestModel {
    pub id: i32,
    pub repository_id: i32,
    pub number: i32,
    pub name: String,
    pub automerge: bool,
    pub step: String,
}

#[derive(Insertable)]
#[table_name = "pull_request"]
pub struct PullRequestCreation<'a> {
    pub repository_id: i32,
    pub number: i32,
    pub name: &'a str,
    pub automerge: bool,
    pub step: &'a str,
}

impl PullRequestModel {
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::pull_request.load::<Self>(conn).map_err(Into::into)
    }

    pub fn get_by_number(conn: &DbConn, repo_id: i32, pr_number: i32) -> Option<Self> {
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

        Self::get_by_number(conn, entry.repository_id, entry.number)
            .ok_or_else(|| eyre!("Error while fetching created pull request"))
    }

    pub fn get_or_create(conn: &DbConn, entry: &PullRequestCreation) -> Result<Self> {
        Self::get_by_number(conn, entry.repository_id, entry.number)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }
}
