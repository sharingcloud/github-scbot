//! Database pull request models.

use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_libs::tracing::error;
use github_scbot_types::{
    common::GhRepository,
    labels::StepLabel,
    pulls::GhPullRequest,
    status::{CheckStatus, QaStatus},
};
use serde::{Deserialize, Serialize};

use super::{
    repository::{IRepositoryDbAdapter, RepositoryModel},
    review::IReviewDbAdapter,
    ReviewModel,
};
use crate::{errors::Result, schema::pull_request};

mod adapter;
mod builder;
pub use adapter::{DummyPullRequestDbAdapter, IPullRequestDbAdapter, PullRequestDbAdapter};
use builder::PullRequestModelBuilder;

/// Pull request model.
#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Identifiable,
    Clone,
    PartialEq,
    Eq,
    AsChangeset,
    Default,
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
pub struct PullRequestCreation {
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

impl From<PullRequestCreation> for PullRequestModel {
    fn from(creation: PullRequestCreation) -> Self {
        Self {
            id: 0,
            repository_id: creation.repository_id,
            number: creation.number,
            creator: creation.creator,
            name: creation.name,
            base_branch: creation.base_branch,
            head_branch: creation.head_branch,
            step: creation.step,
            check_status: creation.check_status,
            qa_status: creation.qa_status,
            needed_reviewers_count: creation.needed_reviewers_count,
            status_comment_id: creation.status_comment_id,
            automerge: creation.automerge,
            wip: creation.wip,
            locked: creation.locked,
            merged: creation.merged,
            closed: creation.closed,
        }
    }
}

impl PullRequestModel {
    /// Create builder.
    pub fn builder<'a>(
        repository: &'a RepositoryModel,
        pr_number: u64,
        creator: &str,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::default(repository, pr_number, creator)
    }

    /// Create builder from model.
    pub fn builder_from_model<'a>(
        repository: &'a RepositoryModel,
        model: &Self,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::from_model(repository, model)
    }

    /// Create builder from GitHub pull request.
    pub fn builder_from_github<'a>(
        repository: &'a RepositoryModel,
        pr: &GhPullRequest,
    ) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::from_github(repository, pr)
    }

    /// Create or update repository and pull request from GitHub objects.
    pub async fn create_or_update_from_github(
        config: Config,
        db_adapter: &dyn IPullRequestDbAdapter,
        repository_db_adapter: &dyn IRepositoryDbAdapter,
        repository: &GhRepository,
        pull_request: &GhPullRequest,
    ) -> Result<(RepositoryModel, Self)> {
        let repository = repository.clone();
        let pull_request = pull_request.clone();

        let repo = RepositoryModel::builder_from_github(&config, &repository)
            .create_or_update(repository_db_adapter)
            .await?;
        let pr = PullRequestModel::builder_from_github(&repo, &pull_request)
            .create_or_update(db_adapter)
            .await?;

        Ok((repo, pr))
    }

    /// Get reviews from a pull request.
    pub async fn get_reviews(
        &self,
        review_db_adapter: &dyn IReviewDbAdapter,
    ) -> Result<Vec<ReviewModel>> {
        review_db_adapter.list_from_pull_request_id(self.id).await
    }

    /// Get pull request repository.
    pub async fn get_repository(
        &self,
        repository_db_adapter: &dyn IRepositoryDbAdapter,
    ) -> Result<RepositoryModel> {
        repository_db_adapter.get_from_id(self.repository_id).await
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
        if let Ok(status) = CheckStatus::try_from(&self.check_status[..]) {
            status
        } else {
            error!(
                pull_request_id = self.id,
                checks_status = %self.check_status,
                message = "Invalid check_status"
            );

            CheckStatus::Skipped
        }
    }

    /// Get QA status enum from database value.
    pub fn get_qa_status(&self) -> QaStatus {
        if let Ok(status) = QaStatus::try_from(&self.qa_status[..]) {
            status
        } else {
            error!(
                pull_request_id = self.id,
                qa_status = %self.qa_status,
                message = "Invalid QA status"
            );

            QaStatus::Skipped
        }
    }

    /// Get step label enum from database value.
    pub fn get_step_label(&self) -> Option<StepLabel> {
        self.step
            .as_ref()
            .and_then(|x| StepLabel::try_from(&x[..]).ok())
    }

    /// Get checks URL for a repository.
    pub fn get_checks_url(&self, repository: &RepositoryModel) -> String {
        return format!(
            "https://github.com/{}/{}/pull/{}/checks",
            repository.owner, repository.name, self.number
        );
    }

    /// Set checks status.
    pub fn set_checks_status(&mut self, checks_status: CheckStatus) {
        self.check_status = checks_status.to_str().to_string();
    }

    /// Set step label.
    pub fn set_step_label(&mut self, step_label: StepLabel) {
        self.step = Some(step_label.to_str().to_string());
    }

    /// Remove step label.
    pub fn remove_step_label(&mut self) {
        self.step = None;
    }

    /// Set QA status.
    pub fn set_qa_status(&mut self, status: QaStatus) {
        self.qa_status = status.to_str().to_string();
    }

    /// Set status comment ID.
    pub fn set_status_comment_id(&mut self, id: u64) {
        self.status_comment_id = id as i32
    }

    /// Remove closed pull requests.
    pub async fn remove_closed_pulls(
        db_adapter: &dyn IPullRequestDbAdapter,
        review_db_adapter: &dyn IReviewDbAdapter,
        repository_id: i32,
    ) -> Result<()> {
        let prs = db_adapter
            .list_closed_pulls_from_repository(repository_id)
            .await?;

        for pr in prs {
            for review in pr.get_reviews(review_db_adapter).await? {
                review_db_adapter.remove(review).await?;
            }

            db_adapter.remove(&pr).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_types::status::{CheckStatus, QaStatus};
    use pretty_assertions::assert_eq;

    use crate::{
        models::{
            pulls::PullRequestDbAdapter, repository::RepositoryDbAdapter, PullRequestModel,
            RepositoryModel,
        },
        tests::using_test_db,
        DatabaseError, Result,
    };

    #[actix_rt::test]
    async fn create_pull_request() -> Result<()> {
        using_test_db("test_db_pulls", |config, pool| async move {
            let repo_db_adapter = RepositoryDbAdapter::new(&pool);
            let db_adapter = PullRequestDbAdapter::new(&pool);
            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(&repo_db_adapter)
                .await
                .unwrap();

            let pr = PullRequestModel::builder(&repo, 1234, "me")
                .name("Toto")
                .create_or_update(&db_adapter)
                .await
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
                    qa_status: QaStatus::Waiting.to_string(),
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
