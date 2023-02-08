use github_scbot_database::DbService;

use crate::Result;

pub struct RemoveAllExternalAccountRightsUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbService,
}

impl<'a> RemoveAllExternalAccountRightsUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_account_rights_delete_all(&self.username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{
        DbService, ExternalAccount, ExternalAccountRight, MemoryDb, Repository,
    };

    use super::RemoveAllExternalAccountRightsUseCase;

    #[actix_rt::test]
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

        RemoveAllExternalAccountRightsUseCase {
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
