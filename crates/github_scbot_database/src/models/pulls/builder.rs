use github_scbot_types::{
    labels::StepLabel,
    pulls::{GhMergeStrategy, GhPullRequest},
    status::{CheckStatus, QaStatus},
};

use super::{IPullRequestDbAdapter, PullRequestCreation, PullRequestModel, PullRequestUpdate};
use crate::{models::RepositoryModel, Result};

#[allow(clippy::option_option)]
#[derive(Default)]
pub struct PullRequestModelBuilder<'a> {
    id: Option<i32>,
    repository: Option<&'a RepositoryModel>,
    pr_number: Option<u64>,
    creator: Option<String>,
    name: Option<String>,
    automerge: Option<bool>,
    step: Option<Option<StepLabel>>,
    check_status: Option<CheckStatus>,
    status_comment_id: Option<u64>,
    qa_status: Option<QaStatus>,
    wip: Option<bool>,
    needed_reviewers_count: Option<u64>,
    locked: Option<bool>,
    merged: Option<bool>,
    base_branch: Option<String>,
    head_branch: Option<String>,
    closed: Option<bool>,
    strategy_override: Option<Option<GhMergeStrategy>>,
}

impl<'a> PullRequestModelBuilder<'a> {
    pub fn with_id(id: i32) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }

    pub fn new(repository: &'a RepositoryModel, pr_number: u64, creator: &str) -> Self {
        Self {
            id: None,
            repository: Some(repository),
            pr_number: Some(pr_number),
            creator: Some(creator.into()),
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
            strategy_override: None,
        }
    }

    pub fn from_model(repository: &'a RepositoryModel, model: &PullRequestModel) -> Self {
        Self {
            id: None,
            repository: Some(repository),
            pr_number: Some(model.number as u64),
            creator: Some(model.creator.clone()),
            name: Some(model.name.clone()),
            automerge: Some(model.automerge),
            step: Some(model.step()),
            check_status: Some(model.check_status()),
            status_comment_id: Some(model.status_comment_id as u64),
            qa_status: Some(model.qa_status()),
            wip: Some(model.wip),
            needed_reviewers_count: Some(model.needed_reviewers_count as u64),
            locked: Some(model.locked),
            merged: Some(model.merged),
            base_branch: Some(model.base_branch.clone()),
            head_branch: Some(model.head_branch.clone()),
            closed: Some(model.closed),
            strategy_override: Some(model.strategy_override()),
        }
    }

    pub fn from_github(repository: &'a RepositoryModel, pr: &GhPullRequest) -> Self {
        Self {
            id: None,
            repository: Some(repository),
            pr_number: Some(pr.number),
            creator: Some(pr.user.login.clone()),
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
            strategy_override: None,
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

    pub fn qa_status(mut self, value: QaStatus) -> Self {
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

    pub fn strategy_override(mut self, value: Option<GhMergeStrategy>) -> Self {
        self.strategy_override = Some(value);
        self
    }

    pub fn build_update(&self) -> PullRequestUpdate {
        let id = self.id.unwrap();

        PullRequestUpdate {
            id,
            creator: self.creator.clone(),
            name: self.name.clone(),
            base_branch: self.base_branch.clone(),
            head_branch: self.head_branch.clone(),
            step: self.step.map(|x| x.map(|x| x.to_str().to_string())),
            check_status: self.check_status.map(|x| x.to_str().to_string()),
            qa_status: self.qa_status.map(|x| x.to_str().to_string()),
            needed_reviewers_count: self.needed_reviewers_count.map(|x| x as i32),
            status_comment_id: self.status_comment_id.map(|x| x as i32),
            automerge: self.automerge,
            wip: self.wip,
            locked: self.locked,
            merged: self.merged,
            closed: self.closed,
            strategy_override: self.strategy_override.map(|x| x.map(|x| x.to_string())),
        }
    }

    pub fn build(&self) -> PullRequestCreation {
        let repo = self.repository.unwrap();
        let pr_number = self.pr_number.unwrap();
        let creator = self.creator.as_ref().unwrap();

        PullRequestCreation {
            repository_id: repo.id(),
            number: pr_number as i32,
            creator: creator.clone(),
            name: self
                .name
                .clone()
                .unwrap_or_else(|| format!("Unnamed PR #{}", pr_number)),
            automerge: self.automerge.unwrap_or_else(|| repo.default_automerge()),
            base_branch: self.base_branch.clone().unwrap_or_else(|| "unknown".into()),
            head_branch: self.head_branch.clone().unwrap_or_else(|| "unknown".into()),
            check_status: self
                .check_status
                .unwrap_or_else(|| {
                    if repo.default_enable_checks() {
                        CheckStatus::Waiting
                    } else {
                        CheckStatus::Skipped
                    }
                })
                .to_string(),
            qa_status: self
                .qa_status
                .unwrap_or_else(|| {
                    if repo.default_enable_qa() {
                        QaStatus::Waiting
                    } else {
                        QaStatus::Skipped
                    }
                })
                .to_string(),
            status_comment_id: self.status_comment_id.unwrap_or(0) as i32,
            needed_reviewers_count: self
                .needed_reviewers_count
                .unwrap_or_else(|| repo.default_needed_reviewers_count() as u64)
                as i32,
            step: self.step.unwrap_or(None).map(|x| x.to_string()),
            wip: self.wip.unwrap_or(false),
            closed: self.closed.unwrap_or(false),
            locked: self.locked.unwrap_or(false),
            merged: self.merged.unwrap_or(false),
            strategy_override: self
                .strategy_override
                .unwrap_or(None)
                .map(|x| x.to_string()),
        }
    }

    pub async fn create_or_update(
        mut self,
        db_adapter: &dyn IPullRequestDbAdapter,
    ) -> Result<PullRequestModel> {
        let repo = self.repository.unwrap();
        let pr_number = self.pr_number.unwrap();

        let handle = match db_adapter
            .get_from_repository_and_number(repo.owner(), repo.name(), pr_number)
            .await
        {
            Ok((mut entry, _)) => {
                self.id = Some(entry.id);
                let update = self.build_update();
                db_adapter.update(&mut entry, update).await?;
                entry
            }
            Err(_) => db_adapter.create(self.build()).await?,
        };
        Ok(handle)
    }
}
