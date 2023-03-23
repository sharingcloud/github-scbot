use github_scbot_database_interface::DbService;
use github_scbot_domain_models::{MergeStrategy, RepositoryPath};

use crate::Result;

pub struct DeterminePullRequestMergeStrategyUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> DeterminePullRequestMergeStrategyUseCase<'a> {
    #[tracing::instrument(
        skip(self),
        fields(repository_path, base_branch, head_branch, default_strategy),
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

#[cfg(test)]
mod tests {
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{MergeRule, Repository, RuleBranch};

    use super::*;

    #[tokio::test]
    async fn no_rule() {
        let db_service = MemoryDb::new();

        let strategy = DeterminePullRequestMergeStrategyUseCase {
            db_service: &db_service,
        }
        .run(&("me", "test").into(), "main", "abcd", MergeStrategy::Merge)
        .await
        .unwrap();

        assert_eq!(strategy, MergeStrategy::Merge);
    }

    #[tokio::test]
    async fn custom_rule() {
        let db_service = MemoryDb::new();
        let repo = db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        db_service
            .merge_rules_create(MergeRule {
                repository_id: repo.id,
                base_branch: RuleBranch::Named("main".into()),
                head_branch: RuleBranch::Named("abcd".into()),
                strategy: MergeStrategy::Squash,
            })
            .await
            .unwrap();

        let strategy = DeterminePullRequestMergeStrategyUseCase {
            db_service: &db_service,
        }
        .run(&("me", "test").into(), "main", "abcd", MergeStrategy::Merge)
        .await
        .unwrap();

        assert_eq!(strategy, MergeStrategy::Squash);
    }
}
