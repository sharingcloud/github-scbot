use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_types::{common::GhRepository, pulls::GhMergeStrategy};

use super::{IRepositoryDbAdapter, RepositoryCreation, RepositoryModel, RepositoryUpdate};
use crate::Result;

#[must_use]
#[derive(Default)]
pub struct RepositoryModelBuilder<'a> {
    id: Option<i32>,
    owner: Option<String>,
    name: Option<String>,
    config: Option<&'a Config>,
    default_strategy: Option<GhMergeStrategy>,
    default_needed_reviewers_count: Option<u64>,
    pr_title_validation_regex: Option<String>,
    manual_interaction: Option<bool>,
    default_automerge: Option<bool>,
    default_enable_qa: Option<bool>,
    default_enable_checks: Option<bool>,
}

impl<'a> RepositoryModelBuilder<'a> {
    pub fn with_id(id: i32) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }

    pub fn new(config: &'a Config, owner: &str, repo_name: &str) -> Self {
        Self {
            id: None,
            owner: Some(owner.into()),
            name: Some(repo_name.into()),
            config: Some(config),
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
            default_automerge: None,
            default_enable_qa: None,
            default_enable_checks: None,
        }
    }

    pub fn from_model(config: &'a Config, model: &RepositoryModel) -> Self {
        Self {
            id: None,
            owner: Some(model.owner.clone()),
            name: Some(model.name.clone()),
            config: Some(config),
            default_strategy: Some(model.default_merge_strategy()),
            default_needed_reviewers_count: Some(model.default_needed_reviewers_count as u64),
            pr_title_validation_regex: Some(model.pr_title_validation_regex.clone()),
            manual_interaction: Some(model.manual_interaction),
            default_automerge: Some(model.default_automerge),
            default_enable_qa: Some(model.default_enable_qa),
            default_enable_checks: Some(model.default_enable_checks),
        }
    }

    pub fn from_github(config: &'a Config, repo: &GhRepository) -> Self {
        Self {
            id: None,
            owner: Some(repo.owner.login.clone()),
            name: Some(repo.name.clone()),
            config: Some(config),
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
            default_automerge: None,
            default_enable_qa: None,
            default_enable_checks: None,
        }
    }

    pub fn pr_title_validation_regex<T: Into<String>>(mut self, regex: T) -> Self {
        self.pr_title_validation_regex = Some(regex.into());
        self
    }

    pub fn default_needed_reviewers_count(mut self, count: u64) -> Self {
        self.default_needed_reviewers_count = Some(count);
        self
    }

    pub fn default_strategy(mut self, strategy: GhMergeStrategy) -> Self {
        self.default_strategy = Some(strategy);
        self
    }

    pub fn manual_interaction(mut self, mode: bool) -> Self {
        self.manual_interaction = Some(mode);
        self
    }

    pub fn default_automerge(mut self, value: bool) -> Self {
        self.default_automerge = Some(value);
        self
    }

    pub fn default_enable_qa(mut self, value: bool) -> Self {
        self.default_enable_qa = Some(value);
        self
    }

    pub fn default_enable_checks(mut self, value: bool) -> Self {
        self.default_enable_checks = Some(value);
        self
    }

    pub fn build_update(&self) -> RepositoryUpdate {
        let id = self.id.unwrap();

        RepositoryUpdate {
            id,
            name: self.name.clone(),
            owner: self.owner.clone(),
            default_strategy: self.default_strategy.map(|x| x.to_string()),
            default_needed_reviewers_count: self.default_needed_reviewers_count.map(|x| x as i32),
            pr_title_validation_regex: self.pr_title_validation_regex.clone(),
            manual_interaction: self.manual_interaction,
            default_automerge: self.default_automerge,
            default_enable_qa: self.default_enable_qa,
            default_enable_checks: self.default_enable_checks,
        }
    }

    pub fn build(&self) -> RepositoryCreation {
        let owner = self.owner.as_ref().unwrap();
        let name = self.name.as_ref().unwrap();
        let config = self.config.unwrap();

        RepositoryCreation {
            owner: owner.to_owned(),
            name: name.to_owned(),
            pr_title_validation_regex: self
                .pr_title_validation_regex
                .clone()
                .unwrap_or_else(|| config.default_pr_title_validation_regex.clone()),
            default_needed_reviewers_count: self
                .default_needed_reviewers_count
                .unwrap_or(config.default_needed_reviewers_count)
                as i32,
            default_strategy: self
                .default_strategy
                .unwrap_or_else(|| {
                    GhMergeStrategy::try_from(&config.default_merge_strategy[..]).unwrap()
                })
                .to_string(),
            manual_interaction: self.manual_interaction.unwrap_or(false),
            default_automerge: self.default_automerge.unwrap_or(false),
            default_enable_qa: self.default_enable_qa.unwrap_or(true),
            default_enable_checks: self.default_enable_checks.unwrap_or(true),
        }
    }

    pub async fn create_or_update(
        mut self,
        db_adapter: &dyn IRepositoryDbAdapter,
    ) -> Result<RepositoryModel> {
        let owner = self.owner.as_ref().unwrap();
        let name = self.name.as_ref().unwrap();

        let handle = match db_adapter.get_from_owner_and_name(owner, name).await {
            Ok(mut entry) => {
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
