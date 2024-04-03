use async_trait::async_trait;
use prbot_models::{Repository, RepositoryPath};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait GetOrCreateRepositoryInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
    ) -> Result<Repository>;
}

#[derive(Component)]
#[shaku(interface = GetOrCreateRepositoryInterface)]
pub(crate) struct GetOrCreateRepository;

#[async_trait]
impl GetOrCreateRepositoryInterface for GetOrCreateRepository {
    #[tracing::instrument(skip(self, ctx), fields(repository_path))]
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        repository_path: &RepositoryPath,
    ) -> Result<Repository> {
        match ctx
            .db_service
            .repositories_get(repository_path.owner(), repository_path.name())
            .await?
        {
            Some(r) => Ok(r),
            None => Ok(ctx
                .db_service
                .repositories_create(
                    Repository {
                        owner: repository_path.owner().into(),
                        name: repository_path.name().into(),
                        ..Default::default()
                    }
                    .with_config(ctx.config),
                )
                .await?),
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;

    use super::*;
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn non_existing() {
        let ctx = CoreContextTest::new();

        let repository = GetOrCreateRepository
            .run(&ctx.as_context(), &("me", "test").into())
            .await
            .unwrap();

        assert_eq!(repository.owner, "me");
        assert_eq!(repository.name, "test");
    }

    #[tokio::test]
    async fn already_existing() {
        let ctx = CoreContextTest::new();
        let original_repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "me".into(),
                name: "test".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let repository = GetOrCreateRepository
            .run(&ctx.as_context(), &("me", "test").into())
            .await
            .unwrap();

        assert_eq!(original_repo, repository);
    }
}
