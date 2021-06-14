use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_types::{common::GhRepository, pulls::GhMergeStrategy};

use super::{IRepositoryDbAdapter, RepositoryCreation, RepositoryModel};
use crate::Result;

#[must_use]
pub struct RepositoryModelBuilder<'a> {
    owner: String,
    name: String,
    config: &'a Config,
    default_strategy: Option<GhMergeStrategy>,
    default_needed_reviewers_count: Option<u64>,
    pr_title_validation_regex: Option<String>,
    manual_interaction: Option<bool>,
}

impl<'a> RepositoryModelBuilder<'a> {
    pub fn default(config: &'a Config, owner: &str, repo_name: &str) -> Self {
        Self {
            owner: owner.into(),
            name: repo_name.into(),
            config,
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
        }
    }

    pub fn from_model(config: &'a Config, model: &RepositoryModel) -> Self {
        Self {
            owner: model.owner.clone(),
            name: model.name.clone(),
            config,
            default_strategy: Some(model.get_default_merge_strategy()),
            default_needed_reviewers_count: Some(model.default_needed_reviewers_count as u64),
            pr_title_validation_regex: Some(model.pr_title_validation_regex.clone()),
            manual_interaction: Some(model.manual_interaction),
        }
    }

    pub fn from_github(config: &'a Config, repo: &GhRepository) -> Self {
        Self {
            owner: repo.owner.login.clone(),
            name: repo.name.clone(),
            config,
            default_strategy: None,
            default_needed_reviewers_count: None,
            pr_title_validation_regex: None,
            manual_interaction: None,
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

    fn build(&self) -> RepositoryCreation {
        RepositoryCreation {
            owner: self.owner.clone(),
            name: self.name.clone(),
            pr_title_validation_regex: self
                .pr_title_validation_regex
                .clone()
                .unwrap_or_else(|| self.config.default_pr_title_validation_regex.clone()),
            default_needed_reviewers_count: self
                .default_needed_reviewers_count
                .unwrap_or(self.config.default_needed_reviewers_count)
                as i32,
            default_strategy: self
                .default_strategy
                .unwrap_or_else(|| {
                    GhMergeStrategy::try_from(&self.config.default_merge_strategy[..]).unwrap()
                })
                .to_string(),
            manual_interaction: self.manual_interaction.unwrap_or(false),
        }
    }

    pub async fn create_or_update(
        self,
        db_adapter: &dyn IRepositoryDbAdapter,
    ) -> Result<RepositoryModel> {
        let mut handle = match db_adapter
            .get_from_owner_and_name(&self.owner, &self.name)
            .await
        {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build()).await?,
        };

        handle.pr_title_validation_regex = match self.pr_title_validation_regex {
            Some(p) => p,
            None => handle.pr_title_validation_regex,
        };
        handle.default_needed_reviewers_count = match self.default_needed_reviewers_count {
            Some(d) => d as i32,
            None => handle.default_needed_reviewers_count,
        };
        handle.default_strategy = match self.default_strategy {
            Some(d) => d.to_string(),
            None => handle.default_strategy,
        };
        handle.manual_interaction = match self.manual_interaction {
            Some(m) => m,
            None => handle.manual_interaction,
        };

        db_adapter.save(&mut handle).await?;
        Ok(handle)
    }
}
