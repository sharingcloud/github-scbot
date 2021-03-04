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
    get_connection,
    schema::{pull_request, repository},
    DbConn, DbPool,
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
    /// PR creator
    pub creator: String,
    /// Pull request title.
    pub name: String,
    /// Base branch.
    pub base_branch: String,
    /// Head branch.
    pub head_branch: String,
    /// Current step label.
    step: Option<String>,
    /// Current check status.
    check_status: String,
    /// QA status.
    qa_status: String,
    /// Needed reviewers count.
    pub needed_reviewers_count: i32,
    /// Status comment ID.
    status_comment_id: i32,
    /// Is automerge enabled?.
    pub automerge: bool,
    /// Is it WIP?.
    pub wip: bool,
    /// Is the PR locked?
    pub locked: bool,
    /// Is the PR merged?
    pub merged: bool,
    /// Is the PR closed?
    pub closed: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "pull_request"]
struct PullRequestCreation {
    pub repository_id: i32,
    pub number: i32,
    pub creator: String,
    pub name: String,
    pub base_branch: String,
    pub head_branch: String,
    pub step: Option<String>,
    pub check_status: String,
    pub qa_status: String,
    pub needed_reviewers_count: i32,
    pub status_comment_id: i32,
    pub automerge: bool,
    pub wip: bool,
    pub locked: bool,
    pub merged: bool,
    pub closed: bool,
}

pub struct PullRequestModelBuilder<'a> {
    repository: &'a RepositoryModel,
    pr_number: u64,
    creator: String,
    name: Option<String>,
    automerge: Option<bool>,
    step: Option<Option<StepLabel>>,
    check_status: Option<CheckStatus>,
    status_comment_id: Option<u64>,
    qa_status: Option<QAStatus>,
    wip: Option<bool>,
    needed_reviewers_count: Option<u64>,
    locked: Option<bool>,
    merged: Option<bool>,
    base_branch: Option<String>,
    head_branch: Option<String>,
    closed: Option<bool>,
}

impl<'a> PullRequestModelBuilder<'a> {
    pub fn default(repository: &'a RepositoryModel, pr_number: u64, creator: &str) -> Self {
        Self {
            repository,
            pr_number,
            creator: creator.into(),
            name: None,
            automerge: None,
            step: None,
            check_status: None,
            status_comment_id: None,
            qa_status: None,
            wip: None,
            needed_reviewers_count: None,
            locked: None,
            merged: None,
            base_branch: None,
            head_branch: None,
            closed: None,
        }
    }

    pub fn from_model(repository: &'a RepositoryModel, model: &PullRequestModel) -> Self {
        Self {
            repository,
            pr_number: model.number as u64,
            creator: model.creator.clone(),
            name: Some(model.name.clone()),
            automerge: Some(model.automerge),
            step: Some(model.get_step_label()),
            check_status: Some(model.get_checks_status()),
            status_comment_id: Some(model.status_comment_id as u64),
            qa_status: Some(model.get_qa_status()),
            wip: Some(model.wip),
            needed_reviewers_count: Some(model.needed_reviewers_count as u64),
            locked: Some(model.locked),
            merged: Some(model.merged),
            base_branch: Some(model.base_branch.clone()),
            head_branch: Some(model.head_branch.clone()),
            closed: Some(model.closed),
        }
    }

    pub fn from_github(repository: &'a RepositoryModel, pr: &GHPullRequest) -> Self {
        Self {
            repository,
            pr_number: pr.number,
            creator: pr.user.login.clone(),
            name: Some(pr.title.clone()),
            automerge: None,
            step: None,
            check_status: None,
            status_comment_id: None,
            qa_status: None,
            wip: Some(pr.draft),
            needed_reviewers_count: None,
            locked: None,
            merged: Some(pr.merged_at.is_some()),
            base_branch: Some(pr.base.reference.clone()),
            head_branch: Some(pr.head.reference.clone()),
            closed: Some(pr.closed_at.is_some()),
        }
    }

    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn automerge(mut self, value: bool) -> Self {
        self.automerge = Some(value);
        self
    }

    pub fn step(mut self, value: Option<StepLabel>) -> Self {
        self.step = Some(value);
        self
    }

    pub fn check_status(mut self, value: CheckStatus) -> Self {
        self.check_status = Some(value);
        self
    }

    pub fn status_comment_id(mut self, id: u64) -> Self {
        self.status_comment_id = Some(id);
        self
    }

    pub fn qa_status(mut self, value: QAStatus) -> Self {
        self.qa_status = Some(value);
        self
    }

    pub fn wip(mut self, value: bool) -> Self {
        self.wip = Some(value);
        self
    }

    pub fn needed_reviewers_count(mut self, value: u64) -> Self {
        self.needed_reviewers_count = Some(value);
        self
    }

    pub fn locked(mut self, value: bool) -> Self {
        self.locked = Some(value);
        self
    }

    pub fn merged(mut self, value: bool) -> Self {
        self.merged = Some(value);
        self
    }

    pub fn base_branch<T: Into<String>>(mut self, branch: T) -> Self {
        self.base_branch = Some(branch.into());
        self
    }

    pub fn head_branch<T: Into<String>>(mut self, branch: T) -> Self {
        self.head_branch = Some(branch.into());
        self
    }

    pub fn closed(mut self, value: bool) -> Self {
        self.closed = Some(value);
        self
    }

    fn build(&self) -> PullRequestCreation {
        PullRequestCreation {
            repository_id: self.repository.id,
            number: self.pr_number as i32,
            creator: self.creator.clone(),
            name: self
                .name
                .clone()
                .unwrap_or_else(|| format!("Unnamed PR #{}", self.pr_number)),
            automerge: self.automerge.unwrap_or(false),
            base_branch: self.base_branch.clone().unwrap_or_else(|| "unknown".into()),
            head_branch: self.head_branch.clone().unwrap_or_else(|| "unknown".into()),
            check_status: self
                .check_status
                .unwrap_or(CheckStatus::Skipped)
                .to_string(),
            qa_status: self.qa_status.unwrap_or(QAStatus::Waiting).to_string(),
            status_comment_id: self.status_comment_id.unwrap_or(0) as i32,
            needed_reviewers_count: self
                .needed_reviewers_count
                .unwrap_or_else(|| self.repository.default_needed_reviewers_count as u64)
                as i32,
            step: self.step.unwrap_or(None).map(|x| x.to_string()),
            wip: self.wip.unwrap_or(false),
            closed: self.closed.unwrap_or(false),
            locked: self.locked.unwrap_or(false),
            merged: self.merged.unwrap_or(false),
        }
    }

    pub fn create_or_update(self, conn: &DbConn) -> Result<PullRequestModel> {
        conn.transaction(|| {
            let mut handle = match PullRequestModel::get_from_repository_and_number(
                conn,
                self.repository,
                self.pr_number,
            ) {
                Ok(entry) => entry,
                Err(_) => {
                    let entry = self.build();
                    PullRequestModel::create(conn, entry)?
                }
            };

            handle.name = match self.name {
                Some(n) => n,
                None => handle.name,
            };
            handle.automerge = match self.automerge {
                Some(a) => a,
                None => handle.automerge,
            };
            handle.base_branch = match self.base_branch {
                Some(b) => b,
                None => handle.base_branch,
            };
            handle.head_branch = match self.head_branch {
                Some(b) => b,
                None => handle.head_branch,
            };
            handle.check_status = match self.check_status {
                Some(c) => c.to_string(),
                None => handle.check_status,
            };
            handle.qa_status = match self.qa_status {
                Some(q) => q.to_string(),
                None => handle.qa_status,
            };
            handle.status_comment_id = match self.status_comment_id {
                Some(s) => s as i32,
                None => handle.status_comment_id,
            };
            handle.needed_reviewers_count = match self.needed_reviewers_count {
                Some(n) => n as i32,
                None => handle.needed_reviewers_count,
            };
            handle.step = match self.step {
                Some(s) => s.map(|x| x.to_string()),
                None => handle.step,
            };
            handle.wip = match self.wip {
                Some(w) => w,
                None => handle.wip,
            };
            handle.closed = match self.closed {
                Some(c) => c,
                None => handle.closed,
            };
            handle.locked = match self.locked {
                Some(l) => l,
                None => handle.locked,
            };
            handle.merged = match self.merged {
                Some(m) => m,
                None => handle.merged,
            };
            handle.save(conn)?;

            Ok(handle)
        })
    }
}

impl PullRequestModel {
    /// Create builder.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository
    /// * `pr_number` - Pull request number
    /// * `creator` - Pull request creator
    pub fn builder<'a>(
        repository: &'a RepositoryModel,
        pr_number: u64,
        creator: &str,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::default(repository, pr_number, creator)
    }

    /// Create builder from model.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository
    /// * `model` - Pull request
    pub fn builder_from_model<'a>(
        repository: &'a RepositoryModel,
        model: &Self,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::from_model(repository, model)
    }

    /// Create builder from GitHub pull request.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository
    /// * `pr` - Pull request
    pub fn builder_from_github<'a>(
        repository: &'a RepositoryModel,
        pr: &GHPullRequest,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::from_github(repository, pr)
    }

    fn create(conn: &DbConn, entry: PullRequestCreation) -> Result<Self> {
        diesel::insert_into(pull_request::table)
            .values(&entry)
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Try to lock the status comment ID field.
    ///
    /// # Arguments
    ///
    /// * `pull_request_id` - Pull request ID
    /// * `pool` - Database pool
    pub async fn try_lock_status_comment_id(pull_request_id: i32, pool: DbPool) -> Result<bool> {
        actix_threadpool::run(move || {
            let conn = get_connection(&pool)?;
            let lock: usize = diesel::update(
                pull_request::table
                    .filter(pull_request::id.eq(pull_request_id))
                    .filter(pull_request::status_comment_id.eq(0)),
            )
            // Use the -1 value as a lock
            .set(pull_request::status_comment_id.eq(-1))
            .execute(&conn)?;

            Ok::<_, DatabaseError>(lock > 0)
        })
        .await
        .map_err(Into::into)
    }

    /// Fetch status comment ID.
    ///
    /// # Arguments
    ///
    /// * `pull_request_id` - Pull request ID
    /// * `pool` - Database pool
    pub async fn fetch_status_comment_id(pull_request_id: i32, pool: DbPool) -> Result<i32> {
        actix_threadpool::run(move || {
            let conn = get_connection(&pool)?;
            let status_id = pull_request::table
                .filter(pull_request::id.eq(pull_request_id))
                .select(pull_request::status_comment_id)
                .get_result(&conn)?;

            Ok::<_, DatabaseError>(status_id)
        })
        .await
        .map_err(Into::into)
    }

    /// List pull requests.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn list(conn: &DbConn) -> Result<Vec<Self>> {
        pull_request::table.load::<Self>(conn).map_err(Into::into)
    }

    /// List pull requests from repository path.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `path` - Repository path
    pub fn list_from_repository_path(conn: &DbConn, path: &str) -> Result<Vec<Self>> {
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
    /// * `repository` - Repository
    /// * `pr_number` - PR number
    pub fn get_from_repository_and_number(
        conn: &DbConn,
        repository: &RepositoryModel,
        pr_number: u64,
    ) -> Result<Self> {
        pull_request::table
            .filter(pull_request::repository_id.eq(repository.id))
            .filter(pull_request::number.eq(pr_number as i32))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownPullRequest(repository.get_path(), pr_number))
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
        pr_number: u64,
    ) -> Result<Self> {
        let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(path)?;

        let (pr, _repo): (Self, Option<RepositoryModel>) = pull_request::table
            .left_join(repository::table.on(repository::id.eq(pull_request::repository_id)))
            .filter(repository::owner.eq(owner))
            .filter(repository::name.eq(name))
            .filter(pull_request::id.eq(pr_number as i32))
            .first(conn)
            .map_err(|_e| DatabaseError::UnknownPullRequest(path.to_string(), pr_number))?;

        Ok(pr)
    }

    /// Get reviews from a pull request.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    pub fn get_reviews(&self, conn: &DbConn) -> Result<Vec<ReviewModel>> {
        ReviewModel::list_from_pull_request_id(conn, self.id)
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
    pub fn get_checks_status(&self) -> CheckStatus {
        CheckStatus::try_from(&self.check_status[..]).unwrap_or_else(|_| {
            panic!(
                "Invalid check_status '{}' stored in database for pull request ID {}",
                self.check_status, self.id
            )
        })
    }

    /// Get QA status enum from database value.
    pub fn get_qa_status(&self) -> QAStatus {
        QAStatus::try_from(&self.qa_status[..]).unwrap_or_else(|_| {
            panic!(
                "Invalid qa_status '{}' stored in database for pull request ID {}",
                self.qa_status, self.id
            )
        })
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

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;
    use github_scbot_types::status::{CheckStatus, QAStatus};
    use pretty_assertions::assert_eq;

    use crate::{
        models::{PullRequestModel, RepositoryModel},
        tests::using_test_db,
        DatabaseError, Result,
    };

    #[actix_rt::test]
    async fn create_pull_request() -> Result<()> {
        let config = Config::from_env();

        using_test_db(&config.clone(), "test_db_pulls", |pool| async move {
            let conn = pool.get()?;
            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(&conn)
                .unwrap();

            let pr = PullRequestModel::builder(&repo, 1234, "me")
                .name("Toto")
                .create_or_update(&conn)
                .unwrap();

            assert_eq!(
                pr,
                PullRequestModel {
                    id: pr.id,
                    repository_id: repo.id,
                    number: 1234,
                    creator: "me".into(),
                    automerge: false,
                    base_branch: "unknown".into(),
                    head_branch: "unknown".into(),
                    check_status: CheckStatus::Skipped.to_string(),
                    closed: false,
                    locked: false,
                    merged: false,
                    name: "Toto".into(),
                    needed_reviewers_count: repo.default_needed_reviewers_count,
                    qa_status: QAStatus::Waiting.to_string(),
                    status_comment_id: 0,
                    step: None,
                    wip: false
                }
            );
            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
