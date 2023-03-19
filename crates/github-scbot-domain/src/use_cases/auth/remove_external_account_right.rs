use github_scbot_database_interface::DbService;
use github_scbot_domain_models::RepositoryPath;

use crate::Result;

pub struct RemoveExternalAccountRightUseCase<'a> {
    pub username: String,
    pub repository_path: RepositoryPath,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> RemoveExternalAccountRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username, self.repository_path))]
    pub async fn run(&mut self) -> Result<()> {
        let (owner, name) = self.repository_path.components();

        self.db_service
            .external_account_rights_delete(owner, name, &self.username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{
        ExternalAccount, ExternalAccountRight, Repository, RepositoryPath,
    };

    use super::RemoveExternalAccountRightUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        db.external_accounts_create(ExternalAccount {
            username: "acc".into(),
            ..Default::default()
        })
        .await?;

        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo.id,
            username: "acc".into(),
        })
        .await?;

        RemoveExternalAccountRightUseCase {
            repository_path: RepositoryPath::new_from_components("owner", "name"),
            username: "acc".into(),
            db_service: &mut db,
        }
        .run()
        .await?;

        assert_eq!(
            db.external_account_rights_get("owner", "name", "acc")
                .await?,
            None
        );

        Ok(())
    }
}
