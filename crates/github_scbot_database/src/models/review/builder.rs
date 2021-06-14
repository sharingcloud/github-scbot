use github_scbot_types::reviews::{GhReview, GhReviewState};

use super::{IReviewDbAdapter, ReviewModel};
use crate::{
    models::{PullRequestModel, RepositoryModel},
    Result,
};

#[must_use]
pub struct ReviewModelBuilder<'a> {
    repo_model: &'a RepositoryModel,
    pr_model: &'a PullRequestModel,
    username: String,
    state: Option<GhReviewState>,
    required: Option<bool>,
    valid: Option<bool>,
}

impl<'a> ReviewModelBuilder<'a> {
    pub fn default<T: Into<String>>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        username: T,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: username.into(),
            state: None,
            required: None,
            valid: None,
        }
    }

    pub fn from_model(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &ReviewModel,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: review.username.clone(),
            state: Some(review.get_review_state()),
            required: Some(review.required),
            valid: Some(review.valid),
        }
    }

    pub fn from_github(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        review: &GhReview,
    ) -> Self {
        Self {
            repo_model,
            pr_model,
            username: review.user.login.clone(),
            state: Some(review.state),
            required: None,
            valid: None,
        }
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.username = username.into();
        self
    }

    pub fn state<T: Into<GhReviewState>>(mut self, state: T) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn required<T: Into<bool>>(mut self, required: T) -> Self {
        self.required = Some(required.into());
        self
    }

    pub fn valid<T: Into<bool>>(mut self, valid: T) -> Self {
        self.valid = Some(valid.into());
        self
    }

    fn build(&self) -> ReviewModel {
        ReviewModel {
            id: -1,
            pull_request_id: self.pr_model.id,
            username: self.username.clone(),
            state: self.state.unwrap_or(GhReviewState::Pending).to_string(),
            required: self.required.unwrap_or(false),
            valid: self.valid.unwrap_or(false),
        }
    }

    pub async fn create_or_update(self, db_adapter: &dyn IReviewDbAdapter) -> Result<ReviewModel> {
        let mut handle = match db_adapter
            .get_from_pull_request_and_username(self.repo_model, self.pr_model, &self.username)
            .await
        {
            Ok(entry) => entry,
            Err(_) => db_adapter.create(self.build().into()).await?,
        };

        handle.state = match self.state {
            Some(s) => s.to_string(),
            None => handle.state,
        };
        handle.required = match self.required {
            Some(r) => r,
            None => handle.required,
        };
        handle.valid = match self.valid {
            Some(v) => v,
            None => handle.valid,
        };

        db_adapter.save(&mut handle).await?;
        Ok(handle)
    }
}
