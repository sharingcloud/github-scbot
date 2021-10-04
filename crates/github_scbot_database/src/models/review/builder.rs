use github_scbot_types::reviews::{GhReview, GhReviewState};

use super::{IReviewDbAdapter, ReviewModel, ReviewUpdate};
use crate::{
    models::{PullRequestModel, RepositoryModel},
    Result,
};

#[must_use]
#[derive(Default)]
pub struct ReviewModelBuilder<'a> {
    id: Option<i32>,
    repo_model: Option<&'a RepositoryModel>,
    pr_model: Option<&'a PullRequestModel>,
    username: Option<String>,
    state: Option<GhReviewState>,
    required: Option<bool>,
    valid: Option<bool>,
}

impl<'a> ReviewModelBuilder<'a> {
    pub fn with_id(id: i32) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }

    pub fn new<T: Into<String>>(
        repo_model: &'a RepositoryModel,
        pr_model: &'a PullRequestModel,
        username: T,
    ) -> Self {
        Self {
            id: None,
            repo_model: Some(repo_model),
            pr_model: Some(pr_model),
            username: Some(username.into()),
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
            id: None,
            repo_model: Some(repo_model),
            pr_model: Some(pr_model),
            username: Some(review.username.clone()),
            state: Some(review.state()),
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
            id: None,
            repo_model: Some(repo_model),
            pr_model: Some(pr_model),
            username: Some(review.user.login.clone()),
            state: Some(review.state),
            required: None,
            valid: None,
        }
    }

    pub fn username<T: Into<String>>(mut self, username: T) -> Self {
        self.username = Some(username.into());
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

    pub fn build_update(&self) -> ReviewUpdate {
        let id = self.id.unwrap();

        ReviewUpdate {
            id,
            username: self.username.clone(),
            state: self.state.map(|x| x.to_string()),
            required: self.required,
            valid: self.valid,
        }
    }

    pub fn build(&self) -> ReviewModel {
        let pr_model = self.pr_model.unwrap();
        let username = self.username.as_ref().unwrap();

        ReviewModel {
            id: -1,
            pull_request_id: pr_model.id(),
            username: username.to_owned(),
            state: self.state.unwrap_or(GhReviewState::Pending).to_string(),
            required: self.required.unwrap_or(false),
            valid: self.valid.unwrap_or(false),
        }
    }

    pub async fn create_or_update(
        mut self,
        db_adapter: &dyn IReviewDbAdapter,
    ) -> Result<ReviewModel> {
        let repo_model = self.repo_model.unwrap();
        let pr_model = self.pr_model.unwrap();
        let username = self.username.as_ref().unwrap();

        let handle = match db_adapter
            .get_from_pull_request_and_username(repo_model, pr_model, username)
            .await
        {
            Ok(mut entry) => {
                self.id = Some(entry.id);
                let update = self.build_update();
                db_adapter.update(&mut entry, update).await?;
                entry
            }
            Err(_) => db_adapter.create(self.build().into()).await?,
        };

        Ok(handle)
    }
}
