//! API handlers

use std::convert::TryInto;

use actix_web::{error, web, Error, HttpResponse};
use serde::Deserialize;

use crate::{
    database::models::{PullRequestModel, RepositoryModel},
    errors::Result,
    webhook::logic::welcome::post_welcome_comment,
};

#[derive(Debug, Deserialize)]
pub struct WelcomeMessageData {
    pub repo_owner: String,
    pub repo_name: String,
    pub pr_number: u64,
    pub pr_author: String,
}

pub async fn welcome_comment(data: web::Json<WelcomeMessageData>) -> Result<HttpResponse, Error> {
    let repo_model = RepositoryModel {
        id: 1,
        name: data.repo_name.clone(),
        owner: data.repo_owner.clone(),
        pr_title_validation_regex: "".to_string(),
        ..Default::default()
    };

    let pr_model = PullRequestModel {
        id: 1,
        repository_id: repo_model.id,
        number: data
            .pr_number
            .try_into()
            .map_err(error::ErrorInternalServerError)?,
        automerge: false,
        check_status: None,
        step: None,
        name: "Test".to_string(),
        status_comment_id: 0,
        qa_status: None,
        wip: false,
        ..Default::default()
    };

    post_welcome_comment(&repo_model, &pr_model, &data.pr_author)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Welcome comment ok."))
}
