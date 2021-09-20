use diesel::prelude::*;
use github_scbot_libs::{async_trait::async_trait, tokio_diesel::AsyncRunQueryDsl};
use github_scbot_utils::Mock;

use super::{MergeRuleCreation, MergeRuleModel, RuleBranch};
use crate::{models::RepositoryModel, schema::merge_rule, DatabaseError, DbPool, Result};

/// Merge rule DB adapter.
#[async_trait]
pub trait IMergeRuleDbAdapter {
    /// Creates a new merge rule entry.
    async fn create(&self, entry: MergeRuleCreation) -> Result<MergeRuleModel>;
    /// Gets a merge rule from branches.
    async fn get_from_branches(
        &self,
        repository: &RepositoryModel,
        base_branch: &RuleBranch,
        head_branch: &RuleBranch,
    ) -> Result<MergeRuleModel>;
    /// Lists merge rules from a repository ID.
    async fn list_from_repository_id(&self, repository_id: i32) -> Result<Vec<MergeRuleModel>>;
    /// Lists existing merge rules.
    async fn list(&self) -> Result<Vec<MergeRuleModel>>;
    /// Remove a specific merge rule.
    async fn remove(&self, entry: MergeRuleModel) -> Result<()>;
    /// Saves and updates a specific merge rule.
    async fn save(&self, entry: &mut MergeRuleModel) -> Result<()>;
}

/// Concrete merge rule DB adapter.
pub struct MergeRuleDbAdapter<'a> {
    pool: &'a DbPool,
}

impl<'a> MergeRuleDbAdapter<'a> {
    /// Creates a new merge rule DB adapter.
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IMergeRuleDbAdapter for MergeRuleDbAdapter<'a> {
    async fn create(&self, entry: MergeRuleCreation) -> Result<MergeRuleModel> {
        diesel::insert_into(merge_rule::table)
            .values(entry)
            .get_result_async(self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn get_from_branches(
        &self,
        repository: &RepositoryModel,
        base_branch: &RuleBranch,
        head_branch: &RuleBranch,
    ) -> Result<MergeRuleModel> {
        let base_branch = base_branch.clone();
        let head_branch = head_branch.clone();
        let repository = repository.clone();

        merge_rule::table
            .filter(merge_rule::repository_id.eq(repository.id))
            .filter(merge_rule::base_branch.eq(base_branch.name()))
            .filter(merge_rule::head_branch.eq(head_branch.name()))
            .first_async(self.pool)
            .await
            .map_err(|_e| {
                DatabaseError::UnknownMergeRule(
                    repository.get_path(),
                    base_branch.name(),
                    head_branch.name(),
                )
            })
    }

    async fn list_from_repository_id(&self, repository_id: i32) -> Result<Vec<MergeRuleModel>> {
        merge_rule::table
            .filter(merge_rule::repository_id.eq(repository_id))
            .get_results_async(self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn list(&self) -> Result<Vec<MergeRuleModel>> {
        merge_rule::table
            .load_async::<MergeRuleModel>(self.pool)
            .await
            .map_err(DatabaseError::from)
    }

    async fn remove(&self, entry: MergeRuleModel) -> Result<()> {
        diesel::delete(merge_rule::table.filter(merge_rule::id.eq(entry.id)))
            .execute_async(self.pool)
            .await?;

        Ok(())
    }

    async fn save(&self, entry: &mut MergeRuleModel) -> Result<()> {
        let copy = entry.clone();

        *entry = diesel::update(merge_rule::table.filter(merge_rule::id.eq(copy.id)))
            .set(copy)
            .get_result_async(self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Dummy merge rule DB adapter.
pub struct DummyMergeRuleDbAdapter {
    /// Create response.
    pub create_response: Mock<Option<Result<MergeRuleModel>>>,
    /// Get from branches response.
    pub get_from_branches_response: Mock<Result<MergeRuleModel>>,
    /// List from repository ID response.
    pub list_from_repository_id_response: Mock<Result<Vec<MergeRuleModel>>>,
    /// List response.
    pub list_response: Mock<Result<Vec<MergeRuleModel>>>,
    /// Remove response.
    pub remove_response: Mock<Result<()>>,
    /// Save response.
    pub save_response: Mock<Result<()>>,
}

impl Default for DummyMergeRuleDbAdapter {
    fn default() -> Self {
        Self {
            create_response: Mock::new(None),
            get_from_branches_response: Mock::new(Ok(MergeRuleModel::default())),
            list_from_repository_id_response: Mock::new(Ok(Vec::new())),
            list_response: Mock::new(Ok(Vec::new())),
            remove_response: Mock::new(Ok(())),
            save_response: Mock::new(Ok(())),
        }
    }
}

impl DummyMergeRuleDbAdapter {
    /// Creates a new dummy merge rule DB adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(unused_variables)]
impl IMergeRuleDbAdapter for DummyMergeRuleDbAdapter {
    async fn create(&self, entry: MergeRuleCreation) -> Result<MergeRuleModel> {
        self.create_response
            .response()
            .map_or_else(|| Ok(entry.into()), |r| r)
    }

    async fn get_from_branches(
        &self,
        repository: &RepositoryModel,
        base_branch: &RuleBranch,
        head_branch: &RuleBranch,
    ) -> Result<MergeRuleModel> {
        self.get_from_branches_response.response()
    }

    async fn list_from_repository_id(&self, repository_id: i32) -> Result<Vec<MergeRuleModel>> {
        self.list_from_repository_id_response.response()
    }

    async fn list(&self) -> Result<Vec<MergeRuleModel>> {
        self.list_response.response()
    }

    async fn remove(&self, entry: MergeRuleModel) -> Result<()> {
        self.remove_response.response()
    }

    async fn save(&self, entry: &mut MergeRuleModel) -> Result<()> {
        self.save_response.response()
    }
}
