//! Auth commands.

use github_scbot_database::models::{
    AccountModel, ExternalAccountModel, IDatabaseAdapter, RepositoryModel,
};

use super::errors::Result;

pub(crate) async fn create_external_account(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    ExternalAccountModel::builder(username)
        .generate_keys()
        .create_or_update(db_adapter.external_account())
        .await?;
    println!("External account '{}' created.", username);

    Ok(())
}

pub(crate) async fn list_external_accounts(db_adapter: &dyn IDatabaseAdapter) -> Result<()> {
    let accounts = db_adapter.external_account().list().await?;
    if accounts.is_empty() {
        println!("No external account found.");
    } else {
        println!("External accounts:");
        for account in accounts {
            println!("- {}", account.username);
        }
    }

    Ok(())
}

pub(crate) async fn remove_external_account(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    let account = db_adapter
        .external_account()
        .get_from_username(username)
        .await?;
    db_adapter.external_account().remove(account).await?;

    println!("External account '{}' removed.", username);

    Ok(())
}

pub(crate) async fn create_external_token(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    let account = db_adapter
        .external_account()
        .get_from_username(username)
        .await?;
    println!("{}", account.generate_access_token()?);

    Ok(())
}

pub(crate) async fn add_account_right(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    let account = db_adapter
        .external_account()
        .get_from_username(&username)
        .await?;

    db_adapter
        .external_account_right()
        .add_right(&account.username, &repo)
        .await?;
    println!(
        "Right added to repository '{}' for account '{}'.",
        repository_path, username
    );

    Ok(())
}

pub(crate) async fn remove_account_right(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    let account = db_adapter
        .external_account()
        .get_from_username(&username)
        .await?;

    db_adapter
        .external_account_right()
        .remove_right(&account.username, &repo)
        .await?;
    println!(
        "Right removed to repository '{}' for account '{}'.",
        repository_path, username
    );

    Ok(())
}

pub(crate) async fn remove_account_rights(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    let account = db_adapter
        .external_account()
        .get_from_username(&username)
        .await?;

    db_adapter
        .external_account_right()
        .remove_rights(&account.username)
        .await?;
    println!("All rights removed from account '{}'.", username);

    Ok(())
}

pub(crate) async fn list_account_rights(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    let account = db_adapter
        .external_account()
        .get_from_username(&username)
        .await?;
    let rights = db_adapter
        .external_account_right()
        .list_rights(&account.username)
        .await?;
    if rights.is_empty() {
        println!("No right found from account '{}'.", username);
    } else {
        println!("Rights from account '{}':", username);
        for right in rights {
            if let Ok(repo) = right.get_repository(db_adapter.repository()).await {
                println!("- {}", repo.get_path());
            }
        }
    }

    Ok(())
}

pub(crate) async fn add_admin_rights(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    AccountModel::builder(username)
        .admin(true)
        .create_or_update(db_adapter.account())
        .await?;

    println!("Account '{}' added/edited with admin rights.", username);

    Ok(())
}

pub(crate) async fn remove_admin_rights(
    db_adapter: &dyn IDatabaseAdapter,
    username: &str,
) -> Result<()> {
    AccountModel::builder(username)
        .admin(false)
        .create_or_update(db_adapter.account())
        .await?;

    println!("Account '{}' added/edited without admin rights.", username);

    Ok(())
}

pub(crate) async fn list_admin_accounts(db_adapter: &dyn IDatabaseAdapter) -> Result<()> {
    let accounts = db_adapter.account().list_admin_accounts().await?;
    if accounts.is_empty() {
        println!("No admin account found.");
    } else {
        println!("Admin accounts:");
        for account in accounts {
            println!("- {}", account.username);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use github_scbot_database::models::DummyDatabaseAdapter;

    use super::*;

    fn arrange() -> DummyDatabaseAdapter {
        DummyDatabaseAdapter::new()
    }

    #[actix_rt::test]
    async fn test_create_external_account() -> Result<()> {
        create_external_account(&arrange(), "test").await
    }

    #[actix_rt::test]
    async fn test_list_external_accounts() -> Result<()> {
        list_external_accounts(&arrange()).await
    }

    #[actix_rt::test]
    async fn test_remove_external_accounts() -> Result<()> {
        remove_external_account(&arrange(), "test").await
    }

    #[actix_rt::test]
    async fn test_create_external_token() -> Result<()> {
        let mut adapter = arrange();
        adapter
            .external_account_adapter
            .get_from_username_response
            .set_response(Ok(ExternalAccountModel::builder("test")
                .generate_keys()
                .build()));

        create_external_token(&adapter, "test").await
    }

    #[actix_rt::test]
    async fn test_add_account_right() -> Result<()> {
        add_account_right(&arrange(), "test", "repo/path").await
    }

    #[actix_rt::test]
    async fn test_remove_account_right() -> Result<()> {
        remove_account_right(&arrange(), "test", "repo/path").await
    }

    #[actix_rt::test]
    async fn test_remove_account_rights() -> Result<()> {
        remove_account_rights(&arrange(), "test").await
    }

    #[actix_rt::test]
    async fn test_list_account_rights() -> Result<()> {
        list_account_rights(&arrange(), "test").await
    }
}
