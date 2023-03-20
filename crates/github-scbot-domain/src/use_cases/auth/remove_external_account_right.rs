use github_scbot_database_interface::DbService;
use github_scbot_domain_models::RepositoryPath;

use crate::Result;

pub struct RemoveExternalAccountRightUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> RemoveExternalAccountRightUseCase<'a> {
    #[tracing::instrument(skip(self), fields(self.username, self.repository_path))]
    pub async fn run(&self, repository_path: &RepositoryPath, username: &str) -> Result<()> {
        let (owner, name) = repository_path.components();

        self.db_service
            .external_account_rights_delete(owner, name, username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database_interface::DbService;
    use github_scbot_database_memory::MemoryDb;
    use github_scbot_domain_models::{ExternalAccount, ExternalAccountRight, Repository};

    use super::RemoveExternalAccountRightUseCase;

    #[tokio::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let db = MemoryDb::new();

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

        RemoveExternalAccountRightUseCase { db_service: &db }
            .run(&("owner", "name").into(), "acc")
            .await?;

        assert_eq!(
            db.external_account_rights_get("owner", "name", "acc")
                .await?,
            None
        );

        Ok(())
    }
}
