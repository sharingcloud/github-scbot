mod errors;
pub(crate) mod exchange;
pub(crate) mod fields;
pub(crate) mod models;
pub(crate) mod utils;

use std::ops::Deref;

pub use exchange::Exchanger;
pub use models::account::{Account, AccountDB, AccountDBImpl, AccountDBImplPool, MockAccountDB};
pub use models::external_account::{
    ExternalAccount, ExternalAccountDB, ExternalAccountDBImpl, ExternalAccountDBImplPool,
    ExternalJwtClaims, MockExternalAccountDB,
};
pub use models::external_account_right::{
    ExternalAccountRight, ExternalAccountRightDB, ExternalAccountRightDBImpl,
    ExternalAccountRightDBImplPool, MockExternalAccountRightDB,
};
pub use models::health::{HealthDB, HealthDBImpl, HealthDBImplPool, MockHealthDB};
pub use models::merge_rule::{
    MergeRule, MergeRuleDB, MergeRuleDBImpl, MergeRuleDBImplPool, MockMergeRuleDB,
};
pub use models::pull_request::{
    MockPullRequestDB, PullRequest, PullRequestDB, PullRequestDBImpl, PullRequestDBImplPool,
};
pub use models::repository::{
    MockRepositoryDB, Repository, RepositoryDB, RepositoryDBImpl, RepositoryDBImplPool,
};
pub use models::required_reviewer::{
    MockRequiredReviewerDB, RequiredReviewer, RequiredReviewerDB, RequiredReviewerDBImpl,
    RequiredReviewerDBImplPool,
};

pub use errors::{DatabaseError, Result};
pub type DbPool = sqlx::postgres::PgPool;

use github_scbot_conf::Config;
use sqlx::PgPool;
use sqlx::{migrate::Migrate, postgres::PgPoolOptions, Acquire};

pub async fn run_migrations<'a, A>(migrator: A) -> Result<()>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations")
        .run(migrator)
        .await
        .map_err(|_| DatabaseError::MigrationError)?;

    Ok(())
}

pub async fn establish_pool_connection(config: &Config) -> Result<DbPool> {
    PgPoolOptions::new()
        .max_connections(config.database_pool_size)
        .connect(&config.database_url)
        .await
        .map_err(DatabaseError::ConnectionError)
}

#[mockall::automock]
pub trait DbService: Send + Sync {
    fn accounts(&self) -> Box<dyn AccountDB>;
    fn external_accounts(&self) -> Box<dyn ExternalAccountDB>;
    fn external_account_rights(&self) -> Box<dyn ExternalAccountRightDB>;
    fn merge_rules(&self) -> Box<dyn MergeRuleDB>;
    fn pull_requests(&self) -> Box<dyn PullRequestDB>;
    fn repositories(&self) -> Box<dyn RepositoryDB>;
    fn required_reviewers(&self) -> Box<dyn RequiredReviewerDB>;
    fn health(&self) -> Box<dyn HealthDB>;
}

pub struct DbServiceImplPool {
    pool: PgPool,
}

impl DbServiceImplPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DbService for DbServiceImplPool {
    fn accounts(&self) -> Box<dyn AccountDB> {
        Box::new(AccountDBImplPool::new(self.pool.clone()))
    }

    fn external_accounts(&self) -> Box<dyn ExternalAccountDB> {
        Box::new(ExternalAccountDBImplPool::new(self.pool.clone()))
    }

    fn external_account_rights(&self) -> Box<dyn ExternalAccountRightDB> {
        Box::new(ExternalAccountRightDBImplPool::new(self.pool.clone()))
    }

    fn merge_rules(&self) -> Box<dyn MergeRuleDB> {
        Box::new(MergeRuleDBImplPool::new(self.pool.clone()))
    }

    fn pull_requests(&self) -> Box<dyn PullRequestDB> {
        Box::new(PullRequestDBImplPool::new(self.pool.clone()))
    }

    fn repositories(&self) -> Box<dyn RepositoryDB> {
        Box::new(RepositoryDBImplPool::new(self.pool.clone()))
    }

    fn required_reviewers(&self) -> Box<dyn RequiredReviewerDB> {
        Box::new(RequiredReviewerDBImplPool::new(self.pool.clone()))
    }

    fn health(&self) -> Box<dyn HealthDB> {
        Box::new(HealthDBImplPool::new(self.pool.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{errors::StdError, utils::use_temporary_db};

    use super::*;

    #[actix_rt::test]
    async fn test_service() {
        use_temporary_db(Config::from_env(), "service-test", |_config, conn| async {
            let db = DbServiceImplPool::new(conn);

            async fn sample(
                repo_db: &mut dyn RepositoryDB,
                pr_db: &mut dyn PullRequestDB,
            ) -> core::result::Result<(), StdError> {
                let _r = repo_db.get("", "").await?;
                let _p = pr_db.get("", "", 1).await?;

                Ok(())
            }

            sample(&mut *db.repositories(), &mut *db.pull_requests()).await?;

            Ok(())
        })
        .await
    }
}
