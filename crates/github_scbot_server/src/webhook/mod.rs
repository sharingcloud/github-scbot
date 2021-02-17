//! Webhook handlers.

mod checks;
mod issues;
mod ping;
mod pulls;
mod reviews;

use std::convert::TryFrom;

use actix_web::{error, web, HttpRequest, HttpResponse, Result as ActixResult};
use github_scbot_conf::Config;
use github_scbot_database::DbConn;
use github_scbot_types::events::EventType;
use tracing::info;

use crate::{
    constants::GITHUB_EVENT_HEADER,
    errors::{Result, ServerError},
    server::AppContext,
    utils::convert_payload_to_string,
};

async fn parse_event(
    config: &Config,
    conn: &DbConn,
    event_type: EventType,
    body: &str,
) -> Result<HttpResponse> {
    match event_type {
        EventType::CheckRun => {
            checks::check_run_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::CheckSuite => {
            checks::check_suite_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::IssueComment => {
            issues::issue_comment_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::Ping => ping::ping_event(
            config,
            conn,
            serde_json::from_str(body).map_err(|e| ServerError::EventParseError(event_type, e))?,
        )
        .await
        .map_err(Into::into),
        EventType::PullRequest => {
            pulls::pull_request_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReview => {
            reviews::review_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReviewComment => {
            reviews::review_comment_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
    }
}

fn extract_event_from_request(req: &HttpRequest) -> Option<EventType> {
    req.headers()
        .get(GITHUB_EVENT_HEADER)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| EventType::try_from(x).ok())
}

pub(crate) async fn event_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    ctx: web::Data<AppContext>,
) -> ActixResult<HttpResponse> {
    // Route event depending on header
    if let Some(event_type) = extract_event_from_request(&req) {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            let conn = ctx.pool.get().map_err(error::ErrorInternalServerError)?;
            info!("Incoming event: {:?}", event_type);

            parse_event(&ctx.config, &conn, event_type, &body)
                .await
                .map_err(Into::into)
        } else {
            let event_type: &str = event_type.into();
            Ok(HttpResponse::BadRequest().body(format!("Bad payload for event '{}'.", event_type)))
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Unhandled event."))
    }
}

/// Configure webhook handlers.
///
/// # Arguments
///
/// * `cfg` - Actix service config
pub fn configure_webhook_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(event_handler)));
}

// #[cfg(test)]
// mod tests {
//     use github_scbot_conf::Config;
//     use github_scbot_database::{establish_single_test_connection, establish_test_connection, models::{PullRequestCreation, PullRequestModel, RepositoryCreation, RepositoryModel}};
//     use github_scbot_types::events::EventType;

//     use crate::ServerError;

//     use super::checks::check_suite_event;

//     fn test_config() -> Config {
//         let mut config = Config::from_env();
//         config.api_disable_client = true;
//         config
//     }

//     #[actix_rt::test]
//     async fn test_check_suite_completed() {
//         let config = test_config();

//         let conn = establish_single_test_connection(&config).unwrap();

//         let repo3 = RepositoryModel::create(&conn, RepositoryCreation {
//             name: "Repo3".to_string(),
//             owner: "Owner".to_string(),
//             ..Default::default()
//         }).unwrap();

//         PullRequestModel::create(&conn, PullRequestCreation {
//             repository_id: repo3.id,
//             number: 1214,
//             ..Default::default()
//         }).unwrap();

//         check_suite_event(
//             &config,
//             &conn,
//             serde_json::from_str(include_str!("../tests/fixtures/check_suite_completed.json"))
//                 .map_err(|e| ServerError::EventParseError(EventType::Ping, e)).unwrap()
//         )
//         .await
//         .unwrap();
//     }
// }
