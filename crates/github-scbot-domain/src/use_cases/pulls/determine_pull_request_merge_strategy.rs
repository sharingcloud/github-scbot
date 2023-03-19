use github_scbot_database_interface::DbService;
use github_scbot_domain_models::MergeStrategy;

use crate::Result;

pub struct DeterminePullRequestMergeStrategyUseCase<'a> {
    pub db_service: &'a mut dyn DbService,
    pub repo_name: &'a str,
    pub repo_owner: &'a str,
    pub base_branch: &'a str,
    pub head_branch: &'a str,
    pub default_strategy: MergeStrategy,
}

impl<'a> DeterminePullRequestMergeStrategyUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.repo_owner, self.repo_name, self.base_branch, self.head_branch, self.default_strategy), ret)]
    pub async fn run(&mut self) -> Result<MergeStrategy> {
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
