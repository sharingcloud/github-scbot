//! API handlers

use actix_web::{web, Error, HttpResponse};
use eyre::Result;
use serde::Deserialize;

use super::comments::post_welcome_comment;

#[derive(Debug, Deserialize)]
pub struct WelcomeMessageData {
    pub repo_owner: String,
    pub repo_name: String,
    pub pr_number: u64,
    pub pr_author: String,
}

pub async fn welcome_comment(data: web::Json<WelcomeMessageData>) -> Result<HttpResponse, Error> {
    post_welcome_comment(
        &data.repo_owner,
        &data.repo_name,
        data.pr_number,
        &data.pr_author,
    )
    .await
    .map_err(|e| HttpResponse::InternalServerError().body(e.to_string()))?;

    Ok(HttpResponse::Ok().body("Welcome comment ok."))
}
