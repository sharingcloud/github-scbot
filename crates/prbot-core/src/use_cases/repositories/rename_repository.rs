use async_trait::async_trait;
use prbot_models::{Repository, RepositoryPath};
use shaku::{Component, Interface};

use crate::{CoreContext, Result};

#[cfg_attr(any(test, feature = "testkit"), mockall::automock)]
#[async_trait]
pub trait RenameRepositoryInterface: Interface {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        current_path: RepositoryPath,
        new_path: RepositoryPath,
    ) -> Result<Option<Repository>>;
}

#[derive(Component)]
#[shaku(interface = RenameRepositoryInterface)]
pub(crate) struct RenameRepository;

#[async_trait]
impl RenameRepositoryInterface for RenameRepository {
    async fn run<'a>(
        &self,
        ctx: &CoreContext<'a>,
        current_path: RepositoryPath,
        new_path: RepositoryPath,
    ) -> Result<Option<Repository>> {
        let repo = ctx
            .db_service
            .repositories_get(current_path.owner(), current_path.name())
            .await?;

        if let Some(mut repo) = repo {
            new_path.owner().clone_into(&mut repo.owner);
            new_path.name().clone_into(&mut repo.name);

            let repo = ctx.db_service.repositories_update(repo).await?;
            Ok(Some(repo))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use prbot_database_interface::DbService;
    use prbot_models::{Repository, RepositoryPath};

    use super::{RenameRepository, RenameRepositoryInterface};
    use crate::context::tests::CoreContextTest;

    #[tokio::test]
    async fn rename_known_repository() {
        let ctx = CoreContextTest::new();
        let repo = ctx
            .db_service
            .repositories_create(Repository {
                owner: "previousowner".into(),
                name: "previousname".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let new_repo = RenameRepository
            .run(
                &ctx.as_context(),
                repo.path(),
                RepositoryPath::new_from_components("owner", "name"),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(repo.id, new_repo.id);
        assert_eq!(new_repo.owner, "owner");
        assert_eq!(new_repo.name, "name");
    }

    #[tokio::test]
    async fn rename_unknown_repository() {
        let ctx = CoreContextTest::new();
        let new_repo = RenameRepository
            .run(
                &ctx.as_context(),
                RepositoryPath::new_from_components("previousowner", "previousowner"),
                RepositoryPath::new_from_components("owner", "name"),
            )
            .await
            .unwrap();

        assert_eq!(new_repo, None);
    }
}
