//! Database repository models

use diesel::prelude::*;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use super::DbConn;
use crate::database::schema::repository::{self, dsl};

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[table_name = "repository"]
pub struct RepositoryModel {
    pub id: i32,
    pub name: String,
    pub owner: String,
}

#[derive(Insertable)]
#[table_name = "repository"]
pub struct RepositoryCreation<'a> {
    pub name: &'a str,
    pub owner: &'a str,
}

impl RepositoryModel {
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::repository.load::<Self>(conn).map_err(Into::into)
    }

    pub fn get_from_id(conn: &DbConn, id: i32) -> Option<Self> {
        dsl::repository.filter(dsl::id.eq(id)).first(conn).ok()
    }

    pub fn get_from_name(conn: &DbConn, name: &str, owner: &str) -> Option<Self> {
        dsl::repository
            .filter(dsl::name.eq(name))
            .filter(dsl::owner.eq(owner))
            .first(conn)
            .ok()
    }

    pub fn create(conn: &DbConn, entry: &RepositoryCreation) -> Result<Self> {
        diesel::insert_into(dsl::repository)
            .values(entry)
            .execute(conn)?;

        Self::get_from_name(conn, entry.name, entry.owner)
            .ok_or_else(|| eyre!("Error while fetching created repository"))
    }

    pub fn get_or_create(conn: &DbConn, entry: &RepositoryCreation) -> Result<Self> {
        Self::get_from_name(conn, entry.name, entry.owner)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }
}
