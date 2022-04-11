use async_trait::async_trait;
use github_scbot_database_macros::SCGetter;
use sqlx::{postgres::PgRow, FromRow, PgConnection, Row};

use crate::{errors::Result, DatabaseError};

#[derive(SCGetter, Debug, Clone, Default, derive_builder::Builder)]
#[builder(default)]
pub struct RequiredReviewer {
    #[get]
    pull_request_id: i32,
    #[get_ref]
    username: String,
}

impl RequiredReviewer {
    pub fn builder() -> RequiredReviewerBuilder {
        RequiredReviewerBuilder::default()
    }
}

impl<'r> FromRow<'r, PgRow> for RequiredReviewer {
    fn from_row(row: &'r PgRow) -> core::result::Result<Self, sqlx::Error> {
        Ok(Self {
            pull_request_id: row.try_get("pull_request_id")?,
            username: row.try_get("username")?,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RequiredReviewerDB {
    async fn create(&mut self, instance: RequiredReviewer) -> Result<RequiredReviewer>;
    async fn list(&mut self, owner: &str, name: &str, number: i32)
        -> Result<Vec<RequiredReviewer>>;
    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        number: i32,
        username: &str,
    ) -> Result<Option<RequiredReviewer>>;
    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        number: i32,
        username: &str,
    ) -> Result<bool>;
}

pub struct RequiredReviewerDBImpl<'a> {
    connection: &'a mut PgConnection,
}

impl<'a> RequiredReviewerDBImpl<'a> {
    pub fn new(connection: &'a mut PgConnection) -> Self {
        Self { connection }
    }

    async fn get_from_pull_request_id(
        &mut self,
        pull_request_id: i32,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                WHERE pull_request_id = $1
                AND username = $2
            "#,
        )
        .bind(pull_request_id)
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

#[async_trait]
impl<'a> RequiredReviewerDB for RequiredReviewerDBImpl<'a> {
    async fn create(&mut self, instance: RequiredReviewer) -> Result<RequiredReviewer> {
        sqlx::query(
            r#"
            INSERT INTO required_reviewer
            (
                pull_request_id,
                username
            )
            VALUES
            (
                $1,
                $2
            );
        "#,
        )
        .bind(instance.pull_request_id)
        .bind(instance.username())
        .execute(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)?;

        self.get_from_pull_request_id(instance.pull_request_id, instance.username())
            .await
            .map(|x| x.unwrap())
    }

    async fn get(
        &mut self,
        owner: &str,
        name: &str,
        number: i32,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
                WHERE username = $4
            "#
        )
        .bind(owner)
        .bind(name)
        .bind(number)
        .bind(username)
        .fetch_optional(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }

    async fn delete(
        &mut self,
        owner: &str,
        name: &str,
        number: i32,
        username: &str,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM required_reviewer
            INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
            INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
            WHERE username = $4
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number)
        .bind(username)
        .execute(&mut *self.connection)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(DatabaseError::SqlError)
    }

    async fn list(
        &mut self,
        owner: &str,
        name: &str,
        number: i32,
    ) -> Result<Vec<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
            "#
        )
        .bind(owner)
        .bind(name)
        .bind(number)
        .fetch_all(&mut *self.connection)
        .await
        .map_err(DatabaseError::SqlError)
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_conf::Config;

    use crate::{
        models::repository::{Repository, RepositoryDB},
        utils::use_temporary_db,
        RepositoryDBImpl,
    };

    use super::{RequiredReviewer, RequiredReviewerDB, RequiredReviewerDBImpl};

    #[actix_rt::test]
    async fn test_db() {
        use_temporary_db(
            Config::from_env(),
            "required-reviewer-test-db",
            |_config, conn| async move {
                let mut conn = conn.acquire().await?;
                let repo = {
                    let mut db = RepositoryDBImpl::new(&mut conn);
                    db.create(Repository::builder().build()?).await?
                };

                sqlx::query(
                    r#"
                INSERT INTO pull_request
                (
                    repository_id,
                    number,
                    qa_status,
                    needed_reviewers_count,
                    status_comment_id,
                    checks_enabled,
                    automerge,
                    locked,
                    strategy_override
                )
                VALUES
                (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9
                )
            "#,
                )
                .bind(repo.id())
                .bind(1i32)
                .bind("skipped")
                .bind(0i32)
                .bind(0i32)
                .bind(false)
                .bind(false)
                .bind(false)
                .bind("")
                .execute(&mut conn)
                .await?;

                let mut db = RequiredReviewerDBImpl::new(&mut conn);
                let _reviewer = db
                    .create(
                        RequiredReviewer::builder()
                            .pull_request_id(1)
                            .username("test".into())
                            .build()?,
                    )
                    .await?;
                assert!(db.get("", "", 1, "nope").await?.is_none());
                assert!(db.get("", "", 1, "test").await?.is_some());

                Ok(())
            },
        )
        .await;
    }
}
