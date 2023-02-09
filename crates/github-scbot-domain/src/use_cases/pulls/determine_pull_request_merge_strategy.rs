use github_scbot_core::types::pulls::GhMergeStrategy;
use github_scbot_database_interface::DbService;

use crate::Result;

pub struct DeterminePullRequestMergeStrategyUseCase<'a> {
    pub db_service: &'a mut dyn DbService,
    pub repo_name: &'a str,
    pub repo_owner: &'a str,
    pub base_branch: &'a str,
    pub head_branch: &'a str,
    pub default_strategy: GhMergeStrategy,
}

impl<'a> DeterminePullRequestMergeStrategyUseCase<'a> {
    pub async fn run(&mut self) -> Result<GhMergeStrategy> {
        match self
            .db_service
            .merge_rules_get(
                self.repo_owner,
                self.repo_name,
                self.base_branch.into(),
                self.head_branch.into(),
            )
            .await?
        {
            Some(r) => Ok(r.strategy),
            None => Ok(self.default_strategy),
        }
    }
}
