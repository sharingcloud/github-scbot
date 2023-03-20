use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{MergeStrategy, RepositoryPath};

use crate::Result;

pub struct DeterminePullRequestMergeStrategyUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> DeterminePullRequestMergeStrategyUseCase<'a> {
    #[tracing::instrument(
        skip(self),
        fields(repo_owner, repo_name, base_branch, head_branch, default_strategy),
        ret
    )]
    pub async fn run(
        &self,
        repository_path: &RepositoryPath,
        base_branch: &str,
        head_branch: &str,
        default_strategy: MergeStrategy,
    ) -> Result<MergeStrategy> {
        match self
            .db_service
            .merge_rules_get(
                repository_path.owner(),
                repository_path.name(),
                base_branch.into(),
                head_branch.into(),
            )
            .await?
        {
            Some(r) => Ok(r.strategy),
            None => Ok(default_strategy),
        }
    }
}
