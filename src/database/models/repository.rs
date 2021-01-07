//! Database repository models

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::DbConn;
use crate::database::errors::{DatabaseError, Result};
use crate::database::schema::repository::{self, dsl};

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
    PartialEq,
    Clone,
    Eq,
)]
#[table_name = "repository"]
pub struct RepositoryModel {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub pr_title_validation_regex: String,
    pub default_needed_reviewers_count: i32,
}

impl Default for RepositoryModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            owner: String::new(),
            pr_title_validation_regex: String::new(),
            default_needed_reviewers_count: 2,
        }
    }
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

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
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
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn update_from_instance(&mut self, conn: &DbConn, other: &Self) -> Result<()> {
        self.pr_title_validation_regex = other.pr_title_validation_regex.clone();
        self.save_changes::<Self>(conn)?;

        Ok(())
    }

    pub fn get_from_path(conn: &DbConn, path: &str) -> Result<Option<Self>> {
        let (owner, name) = Self::extract_name_from_path(path)?;
        Ok(Self::get_from_name(conn, name, owner))
    }

    #[allow(clippy::clippy::needless_pass_by_value)]
    pub fn create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
        diesel::insert_into(dsl::repository)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_name(conn, entry.name, entry.owner).ok_or_else(|| {
            DatabaseError::UnknownRepositoryError(format!("{}/{}", entry.owner, entry.name))
        })
    }

    pub fn get_or_create(conn: &DbConn, entry: RepositoryCreation) -> Result<Self> {
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

        Err(DatabaseError::BadRepositoryPathError(path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{RepositoryCreation, RepositoryModel};
    use crate::{database::establish_single_connection, utils::test_init};

    #[test]
    fn create_repository() {
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

        assert_eq!(repo.id, 1);
        assert_eq!(repo.name, "TestRepo");
        assert_eq!(repo.owner, "me");
    }

    #[test]
    fn list_repositories() {
        test_init();

        let conn = establish_single_connection().unwrap();
        RepositoryModel::create(
            &conn,
            RepositoryCreation {
                name: "TestRepo",
                owner: "me",
            },
        )
        .unwrap();

        RepositoryModel::create(
            &conn,
            RepositoryCreation {
                name: "AnotherRepo",
                owner: "me",
            },
        )
        .unwrap();

        let repos = RepositoryModel::list(&conn).unwrap();
        assert_eq!(repos.len(), 2);
    }
}
