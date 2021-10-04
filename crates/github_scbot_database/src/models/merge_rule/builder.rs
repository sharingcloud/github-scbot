use github_scbot_types::pulls::GhMergeStrategy;

use super::{IMergeRuleDbAdapter, MergeRuleModel, MergeRuleUpdate, RuleBranch};
use crate::{models::RepositoryModel, Result};

#[must_use]
#[derive(Default)]
pub struct MergeRuleBuilder<'a> {
    id: Option<i32>,
    repo_model: Option<&'a RepositoryModel>,
    base_branch: Option<RuleBranch>,
    head_branch: Option<RuleBranch>,
    strategy: Option<GhMergeStrategy>,
}

impl<'a> MergeRuleBuilder<'a> {
    pub fn with_id(id: i32) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }

    pub fn new<T1: Into<RuleBranch>, T2: Into<RuleBranch>>(
        repo_model: &'a RepositoryModel,
        base_branch: T1,
        head_branch: T2,
    ) -> Self {
        Self {
            id: None,
            repo_model: Some(repo_model),
            base_branch: Some(base_branch.into()),
            head_branch: Some(head_branch.into()),
            strategy: None,
        }
    }

    pub fn from_model(repo_model: &'a RepositoryModel, model: &MergeRuleModel) -> Self {
        Self {
            id: None,
            repo_model: Some(repo_model),
            base_branch: Some((&model.base_branch[..]).into()),
            head_branch: Some((&model.head_branch[..]).into()),
            strategy: Some(model.strategy()),
        }
    }

    pub fn strategy(mut self, strategy: GhMergeStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn build_update(&self) -> MergeRuleUpdate {
        let id = self.id.unwrap();

        MergeRuleUpdate {
            id,
            head_branch: self.head_branch.as_ref().map(|x| x.name()),
            base_branch: self.base_branch.as_ref().map(|x| x.name()),
            strategy: self.strategy.map(|x| x.to_string()),
        }
    }

    pub fn build(&self) -> MergeRuleModel {
        let repo_model = self.repo_model.unwrap();
        let base_branch = self.base_branch.as_ref().unwrap();
        let head_branch = self.head_branch.as_ref().unwrap();

        MergeRuleModel {
            id: -1,
            repository_id: repo_model.id(),
            base_branch: match base_branch {
                RuleBranch::Named(named) => named.clone(),
                RuleBranch::Wildcard => "*".into(),
            },
            head_branch: match head_branch {
                RuleBranch::Named(named) => named.clone(),
                RuleBranch::Wildcard => "*".into(),
            },
            strategy: self.strategy.unwrap_or(GhMergeStrategy::Merge).to_string(),
        }
    }

    pub async fn create_or_update(
        mut self,
        db_adapter: &dyn IMergeRuleDbAdapter,
    ) -> Result<MergeRuleModel> {
        let repo_model = self.repo_model.unwrap();
        let base_branch = self.base_branch.as_ref().unwrap();
        let head_branch = self.head_branch.as_ref().unwrap();

        let handle = match db_adapter
            .get_from_branches(repo_model, base_branch, head_branch)
            .await
        {
            Ok(mut entry) => {
                self.id = Some(entry.id);
                let update = self.build_update();
                db_adapter.update(&mut entry, update).await?;
                entry
            }
            Err(_) => db_adapter.create(self.build().into()).await?,
        };

        Ok(handle)
    }
}
