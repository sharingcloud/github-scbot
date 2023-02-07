use github_scbot_database::{DbServiceAll, Repository};

use crate::Result;

pub struct ListExternalAccountRightsUseCase<'a> {
    pub username: &'a str,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> ListExternalAccountRightsUseCase<'a> {
    pub async fn run(&mut self) -> Result<Vec<Repository>> {
        let rights = self
            .db_service
            .external_account_rights_list(self.username)
            .await?;

        let mut repositories = Vec::new();
        if !rights.is_empty() {
            for right in rights {
                repositories.push(
                    self.db_service
                        .repositories_get_from_id(right.repository_id)
                        .await?
                        .unwrap(),
                );
            }
        }

        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use github_scbot_database::{
        DbServiceAll, ExternalAccount, ExternalAccountRight, MemoryDb, Repository,
    };

    use super::ListExternalAccountRightsUseCase;

    #[actix_rt::test]
    async fn run() -> Result<(), Box<dyn Error>> {
        let mut db = MemoryDb::new();

        let repo1 = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        let repo2 = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name2".into(),
                ..Default::default()
            })
            .await?;

        db.external_accounts_create(ExternalAccount {
            username: "acc".into(),
            ..Default::default()
        })
        .await?;

        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo1.id,
            username: "acc".into(),
        })
        .await?;
        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo2.id,
            username: "acc".into(),
        })
        .await?;

        assert_eq!(
            ListExternalAccountRightsUseCase {
                username: "acc",
                db_service: &mut db,
            }
            .run()
            .await?,
            vec![repo1, repo2]
        );

        Ok(())
    }
}
