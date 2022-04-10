//! Database pull request models.

use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_database_macros::SCGetter;
use github_scbot_types::{
    common::GhRepository,
    labels::StepLabel,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::{CheckStatus, QaStatus},
};
use serde::{Deserialize, Serialize};
use tracing::error;

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
    Debug, Deserialize, Serialize, Queryable, Identifiable, Clone, PartialEq, Eq, Default, SCGetter,
)]
#[table_name = "pull_request"]
pub struct PullRequestModel {
    /// ID.
    #[get]
    id: i32,
    /// Repository ID.
    #[get]
    repository_id: i32,
    /// PR number.
    #[get_as(u64)]
    number: i32,
    /// Creator.
    #[get_ref]
    creator: String,
    /// Name.
    #[get_ref]
    name: String,
    /// Base branch.
    #[get_ref]
    base_branch: String,
    /// Head branch.
    #[get_ref]
    head_branch: String,
    step: Option<String>,
    check_status: String,
    qa_status: String,
    /// Needed reviewers count.
    #[get]
    needed_reviewers_count: i32,
    /// Status comment ID.
    #[get_as(u64)]
    status_comment_id: i32,
    /// Automerge.
    #[get]
    automerge: bool,
    /// WIP.
    #[get]
    wip: bool,
    /// Locked.
    #[get]
    locked: bool,
    /// Merged.
    #[get]
    merged: bool,
    /// Closed.
    #[get]
    closed: bool,
    strategy_override: Option<String>,
}

#[derive(Debug, Identifiable, Clone, AsChangeset, Default)]
#[table_name = "pull_request"]
pub struct PullRequestUpdate {
    /// Database ID.
    pub id: i32,
    /// PR creator
    pub creator: Option<String>,
    /// Pull request title.
    pub name: Option<String>,
    /// Base branch.
    pub base_branch: Option<String>,
    /// Head branch.
    pub head_branch: Option<String>,
    /// Current step label.
    pub step: Option<Option<String>>,
    /// Current check status.
    pub check_status: Option<String>,
    /// QA status.
    pub qa_status: Option<String>,
    /// Needed reviewers count.
    pub needed_reviewers_count: Option<i32>,
    /// Status comment ID.
    pub status_comment_id: Option<i32>,
    /// Is automerge enabled?.
    pub automerge: Option<bool>,
    /// Is it WIP?.
    pub wip: Option<bool>,
    /// Is the PR locked?
    pub locked: Option<bool>,
    /// Is the PR merged?
    pub merged: Option<bool>,
    /// Is the PR closed?
    pub closed: Option<bool>,
    /// Merge strategy override.
    pub strategy_override: Option<Option<String>>,
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
    pub strategy_override: Option<String>,
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
            strategy_override: creation.strategy_override,
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
        PullRequestModelBuilder::new(repository, pr_number, creator)
    }

    /// Prepare an update builder.
    pub fn create_update<'a>(&self) -> PullRequestModelBuilder<'a> {
        PullRequestModelBuilder::with_id(self.id)
    }

    /// Apply local update on pull request.
    /// Result will not be in database.
    pub fn apply_local_update(&mut self, update: PullRequestUpdate) {
        if let Some(s) = update.name {
            self.name = s;
        }

        if let Some(s) = update.base_branch {
            self.base_branch = s;
        }

        if let Some(s) = update.head_branch {
            self.base_branch = s;
        }

        if let Some(s) = update.step {
            self.step = s;
        }

        if let Some(s) = update.check_status {
            self.check_status = s;
        }

        if let Some(s) = update.qa_status {
            self.qa_status = s;
        }

        if let Some(s) = update.needed_reviewers_count {
            self.needed_reviewers_count = s;
        }

        if let Some(s) = update.status_comment_id {
            self.status_comment_id = s;
        }

        if let Some(s) = update.automerge {
            self.automerge = s;
        }

        if let Some(s) = update.wip {
            self.wip = s;
        }

        if let Some(s) = update.locked {
            self.locked = s;
        }

        if let Some(s) = update.merged {
            self.merged = s;
        }

        if let Some(s) = update.closed {
            self.closed = s;
        }

        if let Some(s) = update.strategy_override {
            self.strategy_override = s;
        }
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
    pub async fn reviews(
        &self,
        review_db_adapter: &dyn IReviewDbAdapter,
    ) -> Result<Vec<ReviewModel>> {
        review_db_adapter.list_from_pull_request_id(self.id).await
    }

    /// Get pull request repository.
    pub async fn repository(
        &self,
        repository_db_adapter: &dyn IRepositoryDbAdapter,
    ) -> Result<RepositoryModel> {
        repository_db_adapter.get_from_id(self.repository_id).await
    }

    /// Get merge commit title.
    pub fn merge_commit_title(&self) -> String {
        format!("{} (#{})", self.name, self.number())
    }

    /// Get checks status enum from database value.
    pub fn check_status(&self) -> CheckStatus {
        if let Ok(status) = CheckStatus::try_from(&self.check_status) {
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
    pub fn qa_status(&self) -> QaStatus {
        if let Ok(status) = QaStatus::try_from(&self.qa_status) {
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
    pub fn step(&self) -> Option<StepLabel> {
        self.step.as_ref().and_then(|x| StepLabel::try_from(x).ok())
    }

    /// Get strategy override.
    pub fn strategy_override(&self) -> Option<GhMergeStrategy> {
        self.strategy_override
            .as_ref()
            .and_then(|x| GhMergeStrategy::try_from(x).ok())
    }

    /// Get checks URL for a repository.
    pub fn checks_url(&self, owner: &str, name: &str) -> String {
        return format!(
            "https://github.com/{owner}/{name}/pull/{}/checks",
            self.number
        );
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
            for review in pr.reviews(review_db_adapter).await? {
                review_db_adapter.remove(review).await?;
            }

            db_adapter.remove(&pr).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_types::{
        pulls::GhMergeStrategy,
        status::{CheckStatus, QaStatus},
    };
    use pretty_assertions::assert_eq;

    use crate::{
        models::{
            pulls::PullRequestDbAdapter, repository::RepositoryDbAdapter, IPullRequestDbAdapter,
            PullRequestModel, RepositoryModel,
        },
        tests::using_test_db,
        DatabaseError, Result,
    };

    #[actix_rt::test]
    async fn update_pull_request() -> Result<()> {
        using_test_db("test_update_pull_request", |config, pool| async move {
            let repo_db_adapter = RepositoryDbAdapter::new(pool.clone());
            let db_adapter = PullRequestDbAdapter::new(pool.clone());
            let repo = RepositoryModel::builder(&config, "me", "TestRepo")
                .create_or_update(&repo_db_adapter)
                .await
                .unwrap();

            let pr1 = PullRequestModel::builder(&repo, 1234, "me")
                .name("Toto")
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            let mut pr2 = PullRequestModel::builder(&repo, 1234, "me")
                .automerge(true)
                .strategy_override(Some(GhMergeStrategy::Rebase))
                .create_or_update(&db_adapter)
                .await
                .unwrap();

            assert_eq!(pr1.id(), pr2.id());
            assert_eq!(pr2.strategy_override(), Some(GhMergeStrategy::Rebase));

            let update = pr2.create_update().strategy_override(None).build_update();
            db_adapter.update(&mut pr2, update).await?;

            assert_eq!(pr2.strategy_override(), None);

            Ok::<_, DatabaseError>(())
        })
        .await
    }

    #[actix_rt::test]
    async fn create_pull_request() -> Result<()> {
        using_test_db("test_db_pulls", |config, pool| async move {
            let repo_db_adapter = RepositoryDbAdapter::new(pool.clone());
            let db_adapter = PullRequestDbAdapter::new(pool.clone());
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
                    repository_id: repo.id(),
                    number: 1234,
                    creator: "me".into(),
                    automerge: false,
                    base_branch: "unknown".into(),
                    head_branch: "unknown".into(),
                    check_status: CheckStatus::Waiting.to_string(),
                    closed: false,
                    locked: false,
                    merged: false,
                    name: "Toto".into(),
                    needed_reviewers_count: repo.default_needed_reviewers_count(),
                    qa_status: QaStatus::Waiting.to_string(),
                    status_comment_id: 0,
                    step: None,
                    wip: false,
                    strategy_override: None
                }
            );
            Ok::<_, DatabaseError>(())
        })
        .await
    }
}
