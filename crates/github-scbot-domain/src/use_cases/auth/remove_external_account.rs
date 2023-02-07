use github_scbot_database::DbServiceAll;

use crate::Result;

pub struct RemoveExternalAccountUseCase<'a> {
    pub username: String,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> RemoveExternalAccountUseCase<'a> {
    pub async fn run(&mut self) -> Result<()> {
        self.db_service
            .external_accounts_delete(&self.username)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{DbServiceAll, ExternalAccount, MemoryDb};

    use super::RemoveExternalAccountUseCase;

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        db.external_accounts_create(ExternalAccount {
            username: "acc".into(),
            ..Default::default()
        })
        .await?;

        RemoveExternalAccountUseCase {
            username: "acc".into(),
            db_service: &mut db,
        }
        .run()
        .await?;

        assert_eq!(db.external_accounts_get("acc").await?, None);

        Ok(())
    }
}
