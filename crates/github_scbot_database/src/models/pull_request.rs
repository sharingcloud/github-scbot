//! Database pull request models.

use std::convert::TryFrom;

use diesel::prelude::*;
use github_scbot_types::{
    labels::StepLabel,
    status::{CheckStatus, QAStatus},
};
use serde::{Deserialize, Serialize};

use super::{repository::RepositoryModel, ReviewModel};
use crate::{
    errors::{DatabaseError, Result},
    schema::{
        pull_request::{self, dsl},
        repository,
    },
    DbConn,
};

/// Pull request model.
#[derive(
    Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, PartialEq, Eq, AsChangeset,
)]
#[table_name = "pull_request"]
pub struct PullRequestModel {
    /// Database ID.
    pub id: i32,
    /// Repository database ID.
    pub repository_id: i32,
    /// Pull request number.
    number: i32,
    /// Pull request title.
    pub name: String,
    /// Is automerge enabled?.
    pub automerge: bool,
    /// Current step label.
    step: Option<String>,
    /// Current check status.
    check_status: Option<String>,
    /// Status comment ID.
    status_comment_id: i32,
    /// QA status.
    qa_status: Option<String>,
    /// Is it WIP?.
    pub wip: bool,
    /// Needed reviewers count.
    pub needed_reviewers_count: i32,
    /// Is the PR locked?.
    pub locked: bool,
}

impl Default for PullRequestModel {
    fn default() -> Self {
        Self {
            id: -1,
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
            locked: false,
        }
    }
}

/// Pull request creation.
#[derive(Debug, Insertable)]
#[table_name = "pull_request"]
pub struct PullRequestCreation {
    /// Repository database ID.
    pub repository_id: i32,
    /// Pull request number.
    pub number: i32,
    /// Pull request title.
    pub name: String,
    /// Is automerge enabled?.
    pub automerge: bool,
    /// Current step label.
    pub step: Option<String>,
    /// Current check status.
    pub check_status: Option<String>,
    /// Status comment ID.
    pub status_comment_id: i32,
    /// QA status.
    pub qa_status: Option<String>,
    /// Is it WIP?.
    pub wip: bool,
    /// Needed reviewers count.
    pub needed_reviewers_count: i32,
    /// Is the PR locked?.
    pub locked: bool,
}

impl Default for PullRequestCreation {
    fn default() -> Self {
        Self {
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
            locked: false,
        }
    }
}

impl PullRequestModel {
    /// Create pull request from creation entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Pull request creation entry
    pub fn create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        diesel::insert_into(dsl::pull_request)
            .values(&entry)
            .execute(conn)?;

        Self::get_from_repository_id_and_number(conn, entry.repository_id, entry.number).ok_or_else(
            || {
                DatabaseError::UnknownPullRequestError(
                    entry.number as u64,
                    entry.repository_id.to_string(),
                )
            },
        )
    }

    /// List pull requests.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        dsl::pull_request.load::<Self>(conn).map_err(Into::into)
    }

    /// List pull requests for repository path.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    pub fn list_for_repository_path(
        conn: &DbConn,
        path: &str,
    ) -> Result<Vec<(Self, Option<RepositoryModel>)>> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;

        pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .get_results(conn)
            .map_err(Into::into)
    }

    /// Get pull request from repository ID and PR number.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository_id` - Repository ID
    /// * `pr_number` - PR number
    pub fn get_from_repository_id_and_number(
        conn: &DbConn,
        repository_id: i32,
        pr_number: i32,
    ) -> Option<Self> {
        dsl::pull_request
            .filter(dsl::repository_id.eq(repository_id))
            .filter(dsl::number.eq(pr_number))
            .first(conn)
            .ok()
    }

    /// Get pull request from repository path and PR number.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    /// * `pr_number` - PR number
    pub fn get_from_repository_path_and_number(
        conn: &DbConn,
        path: &str,
        pr_number: i32,
    ) -> Result<Option<(Self, Option<RepositoryModel>)>> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;

        Ok(pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .filter(pull_request::id.eq(pr_number))
            .first(conn)
            .ok())
    }

    /// Get or create a pull request from creation entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Pull request creation entry
    pub fn get_or_create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        Self::get_from_repository_id_and_number(conn, entry.repository_id, entry.number)
            .map_or_else(|| Self::create(conn, entry), Ok)
    }

    /// Get reviews from a pull request.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn get_reviews(&self, conn: &DbConn) -> Result<Vec<ReviewModel>> {
        ReviewModel::list_for_pull_request_id(conn, self.id)
    }

    /// Get pull request number as u64, to use with GitHub API.
    pub fn get_number(&self) -> u64 {
        self.number as u64
    }

    /// Get status comment ID.
    pub fn get_status_comment_id(&self) -> u64 {
        self.status_comment_id as u64
    }

    /// Get checks status enum from database value.
    pub fn get_checks_status(&self) -> Option<CheckStatus> {
        self.check_status
            .as_ref()
            .and_then(|x| CheckStatus::try_from(&x[..]).ok())
    }

    /// Get QA status enum from database value.
    pub fn get_qa_status(&self) -> Option<QAStatus> {
        self.qa_status
            .as_ref()
            .and_then(|x| QAStatus::try_from(&x[..]).ok())
    }

    /// Get step label enum from database value.
    pub fn get_step_label(&self) -> Option<StepLabel> {
        self.step
            .as_ref()
            .and_then(|x| StepLabel::try_from(&x[..]).ok())
    }

    /// Get checks URL for a repository.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository
    pub fn get_checks_url(&self, repository: &RepositoryModel) -> String {
        return format!(
            "https://github.com/{}/{}/pull/{}/checks",
            repository.owner, repository.name, self.number
        );
    }

    /// Set checks status.
    ///
    /// # Arguments
    ///
    /// * `checks_status` - Checks status
    pub fn set_checks_status(&mut self, checks_status: CheckStatus) {
        self.check_status = Some(checks_status.to_str().to_string());
    }

    /// Set step label.
    ///
    /// # Arguments
    ///
    /// * `step_label` - Step label
    pub fn set_step_label(&mut self, step_label: StepLabel) {
        self.step = Some(step_label.to_str().to_string());
    }

    /// Set QA status.
    ///
    /// # Arguments
    ///
    /// * `qa_status` - QA status
    pub fn set_qa_status(&mut self, status: QAStatus) {
        self.qa_status = Some(status.to_str().to_string());
    }

    /// Set status comment ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Status comment ID
    pub fn set_status_comment_id(&mut self, id: u64) {
        self.status_comment_id = id as i32
    }

    /// Save model instance to database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn save(&mut self, conn: &DbConn) -> Result<()> {
        self.save_changes::<Self>(conn)?;

        Ok(())
    }
}
