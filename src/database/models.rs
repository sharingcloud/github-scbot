//! Database models

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::pull_request::dsl::pull_request as pr_dsl;
use super::schema::repository::dsl::repository as repo_dsl;
use super::schema::{pull_request, repository};

pub type DbConn = SqliteConnection;

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name = "repository"]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub owner: String,
}

impl Repository {
    pub fn list(conn: &DbConn) -> Vec<Self> {
        repo_dsl
            .load::<Repository>(conn)
            .expect("Error loading repositories.")
    }

    pub fn get_by_name(conn: &DbConn, filter_name: &str, filter_owner: &str) -> Option<Self> {
        use super::schema::repository::dsl::*;

        repo_dsl
            .filter(name.eq(filter_name))
            .filter(owner.eq(filter_owner))
            .first(conn)
            .ok()
    }

    pub fn create(conn: &DbConn, entry: NewRepository) -> Self {
        diesel::insert_into(repo_dsl)
            .values(&entry)
            .execute(conn)
            .expect("Error creating repository.");

        Self::get_by_name(conn, entry.name, entry.owner).expect("Error loading repository.")
    }
}

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name = "pull_request"]
pub struct PullRequest {
    pub id: i32,
    pub repository_id: i32,
    pub number: i32,
    pub name: String,
    pub automerge: bool,
    pub step: String,
}

impl PullRequest {
    pub fn list(conn: &DbConn) -> Vec<Self> {
        pr_dsl
            .load::<PullRequest>(conn)
            .expect("Error loading pull requests.")
    }
}

#[derive(Insertable)]
#[table_name = "repository"]
pub struct NewRepository<'a> {
    pub name: &'a str,
    pub owner: &'a str,
}

#[derive(Insertable)]
#[table_name = "pull_request"]
pub struct NewPullRequest<'a> {
    pub repository_id: i32,
    pub number: i32,
    pub name: &'a str,
    pub automerge: bool,
    pub step: &'a str,
}
