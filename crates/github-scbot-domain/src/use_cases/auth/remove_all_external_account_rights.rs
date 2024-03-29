use github_scbot_database_interface::DbService;

use crate::Result;

pub struct RemoveAllExternalAccountRightsUseCase<'a> {
    pub db_service: &'a dyn DbService,
}

impl<'a> RemoveAllExternalAccountRightsUseCase<'a> {
    #[tracing::instrument(skip(self), fields(username))]
    pub async fn run(&self, username: &str) -> Result<()> {
        self.db_service
            .external_account_rights_delete_all(username)
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

    use super::RemoveAllExternalAccountRightsUseCase;

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

        RemoveAllExternalAccountRightsUseCase { db_service: &db }
            .run("acc")
            .await?;

        assert_eq!(
            db.external_account_rights_get("owner", "name", "acc")
                .await?,
            None
        );

        Ok(())
    }
}
