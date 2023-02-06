use async_trait::async_trait;

use crate::{
    Account, DatabaseError, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest,
    Repository, RequiredReviewer, Result,
};
use github_scbot_core::types::{pulls::GhMergeStrategy, rule_branch::RuleBranch, status::QaStatus};
use sqlx::{PgPool, Row};

use super::interface::DbServiceAll;

pub struct PostgresDb {
    pool: PgPool,
}

impl PostgresDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn external_accounts_get_from_id(
        &mut self,
        username: &str,
        repository_id: u64,
    ) -> Result<Option<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
            WHERE username = $1
            AND repository_id = $2
        "#,
        )
        .bind(username)
        .bind(repository_id as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    async fn merge_rules_get_from_id(&mut self, id: i32) -> Result<Option<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT *
            FROM merge_rule
            WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    async fn pull_requests_get_from_id(&mut self, id: u64) -> Result<Option<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT *
            FROM pull_request
            WHERE id = $1
        "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    async fn required_reviewers_get_from_pull_request_id(
        &mut self,
        pull_request_id: u64,
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
        .bind(pull_request_id as i32)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }
}

#[async_trait]
impl DbServiceAll for PostgresDb {
    #[tracing::instrument(skip(self))]
    async fn accounts_create(&mut self, instance: Account) -> Result<Account> {
        let username: String = sqlx::query(
            r#"
            INSERT INTO account
            (
                username,
                is_admin
            )
            VALUES
            (
                $1,
                $2
            )
            RETURNING username
            ;
        "#,
        )
        .bind(instance.username)
        .bind(instance.is_admin)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.accounts_get(&username).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_update(&mut self, instance: Account) -> Result<Account> {
        let username: String = sqlx::query(
            r#"
            UPDATE account
            SET is_admin = $1
            WHERE username = $2
            RETURNING username;
        "#,
        )
        .bind(instance.is_admin)
        .bind(instance.username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.accounts_get(&username).await.map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_all(&mut self) -> Result<Vec<Account>> {
        sqlx::query_as::<_, Account>(
            r#"
                SELECT *
                FROM account
                ORDER BY username
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_get(&mut self, username: &str) -> Result<Option<Account>> {
        sqlx::query_as::<_, Account>(
            r#"
                SELECT *
                FROM account
                WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_delete(&mut self, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM account
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_list_admins(&mut self) -> Result<Vec<Account>> {
        sqlx::query_as::<_, Account>(
            r#"
                SELECT *
                FROM account
                WHERE is_admin = $1
                ORDER BY username
            "#,
        )
        .bind(true)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn accounts_set_is_admin(&mut self, username: &str, value: bool) -> Result<Account> {
        let username: String = sqlx::query(
            r#"
            UPDATE account
            SET is_admin = $1
            WHERE username = $2
            RETURNING username
        "#,
        )
        .bind(value)
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.accounts_get(&username).await.map(|x| x.unwrap())
    }

    //////////////////////////
    // External account rights

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_create(
        &mut self,
        instance: ExternalAccountRight,
    ) -> Result<ExternalAccountRight> {
        sqlx::query(
            r#"
            INSERT INTO external_account_right
            (
                username,
                repository_id
            ) VALUES (
                $1,
                $2
            )
            RETURNING repository_id;
            "#,
        )
        .bind(&instance.username)
        .bind(instance.repository_id as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?;

        self.external_accounts_get_from_id(&instance.username, instance.repository_id)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_get(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<Option<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
                SELECT external_account_right.*
                FROM external_account_right
                INNER JOIN repository ON (repository.id = repository_id)
                WHERE repository.owner = $1
                AND repository.name = $2
                AND username = $3
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_delete(
        &mut self,
        owner: &str,
        name: &str,
        username: &str,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account_right
            USING repository
            WHERE repository.id = repository_id
            AND repository.owner = $1
            AND repository.name = $2
            AND username = $3
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_delete_all(&mut self, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account_right
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_list(
        &mut self,
        username: &str,
    ) -> Result<Vec<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
            WHERE username = $1
            ORDER BY username, repository_id
        "#,
        )
        .bind(username)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_account_rights_all(&mut self) -> Result<Vec<ExternalAccountRight>> {
        sqlx::query_as::<_, ExternalAccountRight>(
            r#"
            SELECT *
            FROM external_account_right
            ORDER BY username, repository_id
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    ////////////////////
    // External accounts

    #[tracing::instrument(skip(self))]
    async fn external_accounts_create(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount> {
        let username: String = sqlx::query(
            r#"
            INSERT INTO external_account
            (
                username,
                public_key,
                private_key
            ) VALUES (
                $1,
                $2,
                $3
            )
            RETURNING username;
            "#,
        )
        .bind(instance.username)
        .bind(instance.public_key)
        .bind(instance.private_key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.external_accounts_get(&username)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn external_accounts_update(
        &mut self,
        instance: ExternalAccount,
    ) -> Result<ExternalAccount> {
        let username: String = sqlx::query(
            r#"
            UPDATE external_account
            SET public_key = $1,
            private_key = $2
            WHERE username = $3
            RETURNING username;
            "#,
        )
        .bind(instance.public_key)
        .bind(instance.private_key)
        .bind(instance.username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.external_accounts_get(&username)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn external_accounts_get(&mut self, username: &str) -> Result<Option<ExternalAccount>> {
        sqlx::query_as::<_, ExternalAccount>(
            r#"
                SELECT *
                FROM external_account
                WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_accounts_delete(&mut self, username: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM external_account
            WHERE username = $1
        "#,
        )
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_accounts_all(&mut self) -> Result<Vec<ExternalAccount>> {
        sqlx::query_as::<_, ExternalAccount>(
            r#"
            SELECT *
            FROM external_account
            ORDER BY username;
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn external_accounts_set_keys(
        &mut self,
        username: &str,
        public_key: &str,
        private_key: &str,
    ) -> Result<ExternalAccount> {
        let username: String = sqlx::query(
            r#"
            UPDATE external_account
            SET public_key = $1,
            private_key = $2
            WHERE username = $3
            RETURNING username
        "#,
        )
        .bind(public_key)
        .bind(private_key)
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.external_accounts_get(&username)
            .await
            .map(|x| x.unwrap())
    }

    ///////////////
    // Health check

    async fn health_check(&mut self) -> Result<()> {
        sqlx::query("SELECT 1;")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::SqlError { source: e })?;

        Ok(())
    }

    //////////////
    // Merge rules

    #[tracing::instrument(skip(self))]
    async fn merge_rules_create(&mut self, instance: MergeRule) -> Result<MergeRule> {
        let new_id: i32 = sqlx::query(
            r#"
            INSERT INTO merge_rule
            (
                repository_id,
                base_branch,
                head_branch,
                strategy
            )
            VALUES
            (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING id
            ;
        "#,
        )
        .bind(instance.repository_id as i32)
        .bind(instance.base_branch.to_string())
        .bind(instance.head_branch.to_string())
        .bind(instance.strategy.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.merge_rules_get_from_id(new_id)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn merge_rules_update(&mut self, instance: MergeRule) -> Result<MergeRule> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE merge_rule
            SET strategy = $1
            WHERE repository_id = $2
            AND base_branch = $3
            AND head_branch = $4
            RETURNING id
        "#,
        )
        .bind(instance.strategy.to_string())
        .bind(instance.repository_id as i32)
        .bind(instance.base_branch.to_string())
        .bind(instance.head_branch.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.merge_rules_get_from_id(new_id)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn merge_rules_get(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<Option<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT merge_rule.*
            FROM merge_rule
            INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2 AND repository.id = repository_id)
            WHERE base_branch = $3
            AND head_branch = $4;
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(base_branch.to_string())
        .bind(head_branch.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn merge_rules_delete(
        &mut self,
        owner: &str,
        name: &str,
        base_branch: RuleBranch,
        head_branch: RuleBranch,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE
            FROM merge_rule
            USING repository
            WHERE repository.owner = $1
            AND repository.name = $2
            AND repository.id = repository_id
            AND base_branch = $3
            AND head_branch = $4;
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(base_branch.to_string())
        .bind(head_branch.to_string())
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn merge_rules_all(&mut self) -> Result<Vec<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT *
            FROM merge_rule
            ORDER BY repository_id, base_branch, head_branch
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn merge_rules_list(&mut self, owner: &str, name: &str) -> Result<Vec<MergeRule>> {
        sqlx::query_as::<_, MergeRule>(
            r#"
            SELECT merge_rule.*
            FROM merge_rule
            INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2 AND repository.id = repository_id)
            ORDER BY repository_id, base_branch, head_branch
        "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    ////////////////
    // Pull requests

    #[tracing::instrument(skip(self))]
    async fn pull_requests_create(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
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
            RETURNING id;
            "#,
        )
        .bind(instance.repository_id as i32)
        .bind(instance.number as i32)
        .bind(instance.qa_status.to_string())
        .bind(instance.needed_reviewers_count as i32)
        .bind(instance.status_comment_id as i32)
        .bind(instance.checks_enabled)
        .bind(instance.automerge)
        .bind(instance.locked)
        .bind(instance.strategy_override.map(|x| x.to_string()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_update(&mut self, instance: PullRequest) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET qa_status = $1,
            needed_reviewers_count = $2,
            status_comment_id = $3,
            checks_enabled = $4,
            automerge = $5,
            locked = $6,
            strategy_override = $7
            WHERE repository_id = $8
            AND number = $9
            RETURNING id;
            "#,
        )
        .bind(instance.qa_status.to_string())
        .bind(instance.needed_reviewers_count as i32)
        .bind(instance.status_comment_id as i32)
        .bind(instance.checks_enabled)
        .bind(instance.automerge)
        .bind(instance.locked)
        .bind(instance.strategy_override.map(|x| x.to_string()))
        .bind(instance.repository_id as i32)
        .bind(instance.number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Option<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT pull_request.*
            FROM pull_request
            INNER JOIN repository ON (repository_id = repository.id)
            WHERE repository.owner = $1
            AND repository.name = $2
            AND number = $3;
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_delete(&mut self, owner: &str, name: &str, number: u64) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM pull_request
            USING repository
            WHERE repository_id = repository.id
            AND repository.owner = $1
            AND repository.name = $2
            AND number = $3;
            "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_list(&mut self, owner: &str, name: &str) -> Result<Vec<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT pull_request.*
            FROM pull_request
            INNER JOIN repository ON (repository_id = repository.id)
            WHERE repository.owner = $1
            AND repository.name = $2
            ORDER BY repository_id, number
            "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_all(&mut self) -> Result<Vec<PullRequest>> {
        sqlx::query_as::<_, PullRequest>(
            r#"
            SELECT *
            FROM pull_request
            ORDER BY repository_id, number
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_qa_status(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        status: QaStatus,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET qa_status = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(status.to_string())
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        count: u64,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET needed_reviewers_count = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(count as i32)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_status_comment_id(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        id: u64,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET status_comment_id = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(id as i32)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_checks_enabled(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET checks_enabled = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_automerge(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET automerge = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_locked(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        value: bool,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET locked = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn pull_requests_set_strategy_override(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        strategy: Option<GhMergeStrategy>,
    ) -> Result<PullRequest> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE pull_request
            SET strategy_override = $1
            FROM repository
            WHERE repository_id = repository.id
            AND repository.owner = $2
            AND repository.name = $3
            AND number = $4
            RETURNING pull_request.id;
            "#,
        )
        .bind(strategy.map(|x| x.to_string()))
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.pull_requests_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    ///////////////
    // Repositories

    #[tracing::instrument(skip(self))]
    async fn repositories_create(&mut self, instance: Repository) -> Result<Repository> {
        let new_id: i32 = sqlx::query(
            r#"
            INSERT INTO repository
            (
                name,
                owner,
                manual_interaction,
                pr_title_validation_regex,
                default_strategy,
                default_needed_reviewers_count,
                default_automerge,
                default_enable_qa,
                default_enable_checks
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
            RETURNING id
            ;
        "#,
        )
        .bind(instance.name)
        .bind(instance.owner)
        .bind(instance.manual_interaction)
        .bind(instance.pr_title_validation_regex)
        .bind(instance.default_strategy.to_string())
        .bind(instance.default_needed_reviewers_count as i32)
        .bind(instance.default_automerge)
        .bind(instance.default_enable_qa)
        .bind(instance.default_enable_checks)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_update(&mut self, instance: Repository) -> Result<Repository> {
        let new_id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET owner = $1,
            name = $2,
            manual_interaction = $3,
            pr_title_validation_regex = $4,
            default_strategy = $5,
            default_needed_reviewers_count = $6,
            default_automerge = $7,
            default_enable_qa = $8,
            default_enable_checks = $9
            WHERE id = $10
            RETURNING id
            ;
        "#,
        )
        .bind(instance.owner)
        .bind(instance.name)
        .bind(instance.manual_interaction)
        .bind(instance.pr_title_validation_regex)
        .bind(instance.default_strategy.to_string())
        .bind(instance.default_needed_reviewers_count as i32)
        .bind(instance.default_automerge)
        .bind(instance.default_enable_qa)
        .bind(instance.default_enable_checks)
        .bind(instance.id as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(new_id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_all(&mut self) -> Result<Vec<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
                SELECT *
                FROM repository
                ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_get(&mut self, owner: &str, name: &str) -> Result<Option<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
            SELECT *
            FROM repository
            WHERE owner = $1
            AND name = $2
        "#,
        )
        .bind(owner)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_get_from_id(&mut self, id: u64) -> Result<Option<Repository>> {
        sqlx::query_as::<_, Repository>(
            r#"
            SELECT *
            FROM repository
            WHERE id = $1
        "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_delete(&mut self, owner: &str, name: &str) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM repository
            WHERE owner = $1 AND name = $2
        "#,
        )
        .bind(owner)
        .bind(name)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_manual_interaction(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET manual_interaction = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_pr_title_validation_regex(
        &mut self,
        owner: &str,
        name: &str,
        value: &str,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET pr_title_validation_regex = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_default_strategy(
        &mut self,
        owner: &str,
        name: &str,
        strategy: GhMergeStrategy,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_strategy = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(strategy.to_string())
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_default_needed_reviewers_count(
        &mut self,
        owner: &str,
        name: &str,
        count: u64,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_needed_reviewers_count = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(count as i32)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_default_automerge(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_automerge = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_default_enable_qa(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_enable_qa = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn repositories_set_default_enable_checks(
        &mut self,
        owner: &str,
        name: &str,
        value: bool,
    ) -> Result<Repository> {
        let id: i32 = sqlx::query(
            r#"
            UPDATE repository
            SET default_enable_checks = $1
            WHERE owner = $2
            AND name = $3
            RETURNING id
        "#,
        )
        .bind(value)
        .bind(owner)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?
        .get(0);

        self.repositories_get_from_id(id as u64)
            .await
            .map(|x| x.unwrap())
    }

    /////////////////////
    // Required reviewers

    #[tracing::instrument(skip(self))]
    async fn required_reviewers_create(
        &mut self,
        instance: RequiredReviewer,
    ) -> Result<RequiredReviewer> {
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
        .bind(instance.pull_request_id as i32)
        .bind(instance.username())
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })?;

        self.required_reviewers_get_from_pull_request_id(
            instance.pull_request_id,
            instance.username(),
        )
        .await
        .map(|x| x.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn required_reviewers_list(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
    ) -> Result<Vec<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
                r#"
                    SELECT required_reviewer.*
                    FROM required_reviewer
                    INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                    INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
                    ORDER BY pull_request_id, username
                "#
            )
            .bind(owner)
            .bind(name)
            .bind(number as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn required_reviewers_get(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<Option<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT required_reviewer.*
                FROM required_reviewer
                INNER JOIN repository ON (repository.owner = $1 AND repository.name = $2)
                INNER JOIN pull_request ON (pull_request.repository_id = repository.id AND pull_request.number = $3 AND required_reviewer.pull_request_id = pull_request.id)
                WHERE username = $4
            "#
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn required_reviewers_delete(
        &mut self,
        owner: &str,
        name: &str,
        number: u64,
        username: &str,
    ) -> Result<bool> {
        sqlx::query(
            r#"
            DELETE FROM required_reviewer
            USING repository, pull_request
            WHERE repository.owner = $1
            AND repository.name = $2
            AND pull_request.repository_id = repository.id
            AND pull_request.number = $3
            AND required_reviewer.pull_request_id = pull_request.id
            AND username = $4
        "#,
        )
        .bind(owner)
        .bind(name)
        .bind(number as i32)
        .bind(username)
        .execute(&self.pool)
        .await
        .map(|x| x.rows_affected() > 0)
        .map_err(|e| DatabaseError::SqlError { source: e })
    }

    #[tracing::instrument(skip(self))]
    async fn required_reviewers_all(&mut self) -> Result<Vec<RequiredReviewer>> {
        sqlx::query_as::<_, RequiredReviewer>(
            r#"
                SELECT *
                FROM required_reviewer
                ORDER BY pull_request_id, username
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::SqlError { source: e })
    }
}
