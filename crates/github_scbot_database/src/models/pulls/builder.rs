use github_scbot_types::{
    labels::StepLabel,
    pulls::GhPullRequest,
    status::{CheckStatus, QaStatus},
};

use super::{IPullRequestDbAdapter, PullRequestCreation, PullRequestModel};
use crate::{models::RepositoryModel, Result};

#[allow(clippy::option_option)]
pub struct PullRequestModelBuilder<'a> {
    repository: &'a RepositoryModel,
    pr_number: u64,
    creator: String,
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

    pub fn from_github(repository: &'a RepositoryModel, pr: &GhPullRequest) -> Self {
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

    pub fn build(&self) -> PullRequestCreation {
        PullRequestCreation {
            repository_id: self.repository.id,
            number: self.pr_number as i32,
            creator: self.creator.clone(),
            name: self
                .name
                .clone()
                .unwrap_or_else(|| format!("Unnamed PR #{}", self.pr_number)),
            automerge: self.automerge.unwrap_or(self.repository.default_automerge),
            base_branch: self.base_branch.clone().unwrap_or_else(|| "unknown".into()),
            head_branch: self.head_branch.clone().unwrap_or_else(|| "unknown".into()),
            check_status: self
                .check_status
                .unwrap_or_else(|| {
                    if self.repository.default_enable_checks {
                        CheckStatus::Waiting
                    } else {
                        CheckStatus::Skipped
                    }
                })
                .to_string(),
            qa_status: self
                .qa_status
                .unwrap_or_else(|| {
                    if self.repository.default_enable_qa {
                        QaStatus::Waiting
                    } else {
                        QaStatus::Skipped
                    }
                })
                .to_string(),
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

    pub async fn create_or_update(
        self,
        db_adapter: &dyn IPullRequestDbAdapter,
    ) -> Result<PullRequestModel> {
        let mut handle = match db_adapter
            .get_from_repository_and_number(self.repository, self.pr_number)
            .await
        {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build()).await?,
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

        db_adapter.save(&mut handle).await?;
        Ok(handle)
    }
}
