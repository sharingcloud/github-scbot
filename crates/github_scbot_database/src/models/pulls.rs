//! Database pull request models.

use std::convert::TryFrom;

use diesel::prelude::*;
use github_scbot_types::{
    labels::StepLabel,
    pulls::GHPullRequest,
    status::{CheckStatus, QAStatus},
};
use serde::{Deserialize, Serialize};

use super::{repository::RepositoryModel, ReviewModel};
use crate::{
    errors::{DatabaseError, Result},
    schema::{pull_request, repository},
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
    check_status: String,
    /// Status comment ID.
    status_comment_id: i32,
    /// QA status.
    qa_status: String,
    /// Is it WIP?.
    pub wip: bool,
    /// Needed reviewers count.
    pub needed_reviewers_count: i32,
    /// Is the PR locked?
    pub locked: bool,
    /// Is the PR merged?
    pub merged: bool,
    /// Base branch.
    pub base_branch: String,
    /// Head branch.
    pub head_branch: String,
    /// Is the PR closed?
    pub closed: bool,
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
    pub check_status: String,
    /// Status comment ID.
    pub status_comment_id: i32,
    /// QA status.
    pub qa_status: String,
    /// Is it WIP?.
    pub wip: bool,
    /// Needed reviewers count.
    pub needed_reviewers_count: i32,
    /// Is the PR locked?
    pub locked: bool,
    /// Is the PR merged?
    pub merged: bool,
    /// Base branch.
    pub base_branch: String,
    /// Head branch.
    pub head_branch: String,
    /// Is the PR closed?
    pub closed: bool,
}

impl PullRequestCreation {
    /// Creates from upstream pull request.
    ///
    /// # Arguments
    ///
    /// * `upstream` - GitHub pull request
    /// * `repository` - Repository
    pub fn from_upstream(upstream: &GHPullRequest, repository: &RepositoryModel) -> Self {
        Self {
            name: upstream.title.clone(),
            number: upstream.number as i32,
            base_branch: upstream.base.reference.clone(),
            head_branch: upstream.head.reference.clone(),
            ..Self::from_repository(repository)
        }
    }

    /// Creates from repository.
    pub fn from_repository(repository: &RepositoryModel) -> Self {
        Self {
            number: -1,
            repository_id: repository.id,
            name: String::new(),
            automerge: false,
            step: None,
            check_status: CheckStatus::Waiting.to_str().into(),
            status_comment_id: 0,
            qa_status: QAStatus::Waiting.to_str().into(),
            wip: false,
            needed_reviewers_count: repository.default_needed_reviewers_count,
            locked: false,
            merged: false,
            base_branch: String::new(),
            head_branch: String::new(),
            closed: false,
        }
    }
}

impl PullRequestModel {
    /// Create default pull request.
    pub fn from_repository(repository: &RepositoryModel) -> Self {
        Self {
            id: -1,
            repository_id: 0,
            number: 0,
            name: String::new(),
            automerge: false,
            step: None,
            check_status: CheckStatus::Waiting.to_str().into(),
            status_comment_id: 0,
            qa_status: QAStatus::Waiting.to_str().into(),
            wip: false,
            needed_reviewers_count: repository.default_needed_reviewers_count,
            locked: false,
            merged: false,
            base_branch: String::new(),
            head_branch: String::new(),
            closed: false,
        }
    }

    /// Create pull request from creation entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Pull request creation entry
    pub fn create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        diesel::insert_into(pull_request::table)
            .values(&entry)
            .execute(conn)?;

        match Self::get_from_repository_id_and_number(conn, entry.repository_id, entry.number) {
            Ok(e) => Ok(e),
            Err(_) => Err(DatabaseError::UnknownPullRequest(
                entry.repository_id.to_string(),
                entry.number,
            )),
        }
    }

    /// List pull requests.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        pull_request::table.load::<Self>(conn).map_err(Into::into)
    }

    /// List pull requests for repository path.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    pub fn list_for_repository_path(conn: &DbConn, path: &str) -> Result<Vec<Self>> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;

        let values: Vec<(Self, Option<RepositoryModel>)> = pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .get_results(conn)?;

        Ok(values.into_iter().map(|(pr, _repo)| pr).collect())
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
    ) -> Result<Self> {
        pull_request::table
            .filter(pull_request::repository_id.eq(repository_id))
            .filter(pull_request::number.eq(pr_number))
            .first(conn)
            .map_err(|_e| {
                DatabaseError::UnknownPullRequest(format!("<ID {}>", repository_id), pr_number)
            })
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
    ) -> Result<Self> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;

        let (pr, _repo): (Self, Option<RepositoryModel>) = pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .filter(pull_request::id.eq(pr_number))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownPullRequest(path.to_string(), pr_number))?;

        Ok(pr)
    }

    /// Get or create a pull request from creation entry.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `entry` - Pull request creation entry
    pub fn get_or_create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        match Self::get_from_repository_id_and_number(conn, entry.repository_id, entry.number) {
            Ok(v) => Ok(v),
            Err(_) => Self::create(conn, entry),
        }
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

    /// Get merge commit title.
    pub fn get_merge_commit_title(&self) -> String {
        format!("{} (#{})", self.name, self.get_number())
    }

    /// Get status comment ID.
    pub fn get_status_comment_id(&self) -> u64 {
        self.status_comment_id as u64
    }

    /// Get checks status enum from database value.
    pub fn get_checks_status(&self) -> Result<CheckStatus> {
        CheckStatus::try_from(&self.check_status[..]).map_err(Into::into)
    }

    /// Get QA status enum from database value.
    pub fn get_qa_status(&self) -> Result<QAStatus> {
        QAStatus::try_from(&self.qa_status[..]).map_err(Into::into)
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
        self.check_status = checks_status.to_str().to_string();
    }

    /// Set step label.
    ///
    /// # Arguments
    ///
    /// * `step_label` - Step label
    pub fn set_step_label(&mut self, step_label: StepLabel) {
        self.step = Some(step_label.to_str().to_string());
    }

    /// Remove step label.
    pub fn remove_step_label(&mut self) {
        self.step = None;
    }

    /// Set QA status.
    ///
    /// # Arguments
    ///
    /// * `qa_status` - QA status
    pub fn set_qa_status(&mut self, status: QAStatus) {
        self.qa_status = status.to_str().to_string();
    }

    /// Set status comment ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Status comment ID
    pub fn set_status_comment_id(&mut self, id: u64) {
        self.status_comment_id = id as i32
    }

    /// Set attributes from upstream pull request.
    ///
    /// # Arguments
    ///
    /// * `upstream_pr` - GitHub Pull request
    pub fn set_from_upstream(&mut self, upstream_pr: &GHPullRequest) {
        self.name = upstream_pr.title.clone();
        self.wip = upstream_pr.draft;
        self.merged = upstream_pr.merged_at.is_some();
        self.closed = upstream_pr.closed_at.is_some();
        self.base_branch = upstream_pr.base.reference.clone();
        self.head_branch = upstream_pr.head.reference.clone();
    }

    /// Remove closed pull requests.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository_id` - Repository ID
    pub fn remove_closed_pulls(conn: &DbConn, repository_id: i32) -> Result<()> {
        let prs = Self::list_closed_pulls(conn, repository_id)?;

        for pr in prs {
            for review in pr.get_reviews(conn)? {
                review.remove(conn)?;
            }

            pr.remove(conn)?;
        }

        Ok(())
    }

    /// List closed pull requests.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `repository_id` - Repository ID
    pub fn list_closed_pulls(conn: &DbConn, repository_id: i32) -> Result<Vec<Self>> {
        pull_request::table
            .filter(pull_request::repository_id.eq(repository_id))
            .filter(pull_request::closed.eq(true))
            .get_results(conn)
            .map_err(Into::into)
    }

    /// Remove pull request.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn remove(&self, conn: &DbConn) -> Result<()> {
        diesel::delete(pull_request::table.filter(pull_request::id.eq(self.id))).execute(conn)?;

        Ok(())
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
