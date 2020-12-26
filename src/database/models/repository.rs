//! Database repository models

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::DbConn;
use crate::database::schema::repository::{self, dsl};
use crate::errors::{BotError, Result};

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Identifiable, AsChangeset)]
#[table_name = "repository"]
pub struct RepositoryModel {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub pr_title_validation_regex: String,
}

#[derive(Insertable)]
#[table_name = "repository"]
pub struct RepositoryCreation<'a> {
    pub name: &'a str,
    pub owner: &'a str,
}

impl RepositoryModel {
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::repository
            .load::<Self>(conn)
            .map_err(|e| BotError::DBError(e.to_string()))
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

    pub fn update_title_pr_validation_regex(&mut self, conn: &DbConn, regex: &str) -> Result<()> {
        self.pr_title_validation_regex = regex.to_owned();
        self.save_changes::<Self>(conn)
            .map_err(|e| BotError::DBError(e.to_string()))?;

        Ok(())
    }

    pub fn get_from_path(conn: &DbConn, path: &str) -> Result<Option<Self>> {
        let (owner, name) = Self::extract_name_from_path(path)?;
        Ok(Self::get_from_name(conn, name, owner))
    }

    pub fn create(conn: &DbConn, entry: &RepositoryCreation) -> Result<Self> {
        diesel::insert_into(dsl::repository)
            .values(entry)
            .execute(conn)
            .map_err(|e| BotError::DBError(e.to_string()))?;

        Self::get_from_name(conn, entry.name, entry.owner)
            .ok_or_else(|| BotError::DBError("Error while fetching created repository".to_string()))
    }

    pub fn get_or_create(conn: &DbConn, entry: &RepositoryCreation) -> Result<Self> {
        Self::get_from_name(conn, entry.name, entry.owner)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }

    pub fn extract_name_from_path(path: &str) -> Result<(&str, &str)> {
        let mut split = path.split_terminator('/');
        let owner = split.next();
        let name = split.next();

        if let Some(owner) = owner {
            if let Some(name) = name {
                return Ok((owner, name));
            }
        }

        Err(BotError::FormatError(format!(
            "Badly formatted repository path: {}",
            path
        )))
    }
}
