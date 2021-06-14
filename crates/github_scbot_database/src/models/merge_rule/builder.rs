use github_scbot_types::pulls::GhMergeStrategy;

use super::{IMergeRuleDbAdapter, MergeRuleModel, RuleBranch};
use crate::{models::RepositoryModel, Result};

#[must_use]
pub struct MergeRuleBuilder<'a> {
    repo_model: &'a RepositoryModel,
    base_branch: RuleBranch,
    head_branch: RuleBranch,
    strategy: Option<GhMergeStrategy>,
}

impl<'a> MergeRuleBuilder<'a> {
    pub fn default<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        repo_model: &'a RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> Self {
        Self {
            repo_model,
            base_branch: base_branch.into(),
            head_branch: head_branch.into(),
            strategy: None,
        }
    }

    pub fn from_model(repo_model: &'a RepositoryModel, model: &MergeRuleModel) -> Self {
        Self {
            repo_model,
            base_branch: (&model.base_branch[..]).into(),
            head_branch: (&model.head_branch[..]).into(),
            strategy: Some(model.get_strategy()),
        }
    }

    pub fn strategy(mut self, strategy: GhMergeStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn build(&self) -> MergeRuleModel {
        MergeRuleModel {
            id: -1,
            repository_id: self.repo_model.id,
            base_branch: match self.base_branch.clone() {
                RuleBranch::Named(named) => named,
                RuleBranch::Wildcard => "*".into(),
            },
            head_branch: match self.head_branch.clone() {
                RuleBranch::Named(named) => named,
                RuleBranch::Wildcard => "*".into(),
            },
            strategy: self.strategy.unwrap_or(GhMergeStrategy::Merge).to_string(),
        }
    }

    pub async fn create_or_update(
        self,
        db_adapter: &dyn IMergeRuleDbAdapter,
    ) -> Result<MergeRuleModel> {
        let mut handle = match db_adapter
            .get_from_branches(self.repo_model, &self.base_branch, &self.head_branch)
            .await
        {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build().into()).await?,
        };

        handle.base_branch = self.base_branch.name();
        handle.head_branch = self.head_branch.name();
        handle.strategy = match self.strategy {
            Some(s) => s.to_string(),
            None => handle.strategy,
        };
        db_adapter.save(&mut handle).await?;

        Ok(handle)
    }
}
